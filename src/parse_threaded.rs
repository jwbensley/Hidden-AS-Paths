use crate::args::cli_args::CliArgs;
use crate::args::cli_args::RibsSource;
use crate::clients::mrt_archives::download_ribs_for_day;
use crate::data::paths::Paths;
use crate::data::record_data::RecordData;
use crate::parse_mrt::{get_peer_table, parse_mrt_entry};
use crate::types::rib::RibFile;
use bgpkit_parser::BgpkitParser;
use log::{debug, info};
use rayon::ThreadPoolBuilder;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rayon::slice::ParallelSliceMut;
use std::sync::atomic::AtomicU32;
use std::sync::{Arc, RwLock};

pub fn get_paths(args: &CliArgs) -> Paths {
    ThreadPoolBuilder::new()
        .num_threads((args.threads).try_into().unwrap())
        .build_global()
        .unwrap();

    match args.ribs_source {
        // Download MRT files and then parse them using one thread per file
        RibsSource::Download(_) => {
            let rib_files = download_ribs_for_day(args.get_ribs_ymd(), args.get_ribs_path());
            init_parallel_parsing(&rib_files)
        }

        // Parse a single existing file split across multiple threads
        RibsSource::File(_) => {
            let rib_files = Vec::from([RibFile::new(String::new(), args.get_rib_file().clone())]);
            init_parallel_parsing(&rib_files)
        }

        // Parse multiple existing files using one thread per file
        RibsSource::Files(_) => {
            let rib_files: Vec<RibFile> = args
                .get_rib_files()
                .iter()
                .map(|filename| RibFile {
                    url: String::new(),
                    filename: filename.clone(),
                })
                .collect();

            init_parallel_parsing(&rib_files)
        }
    }
}

/// Setup and call parallel parsing of RIB files
pub fn init_parallel_parsing(rib_files: &Vec<RibFile>) -> Paths {
    info!("Going to parse {} RIB files", rib_files.len());
    debug!(
        "{:?}",
        rib_files
            .iter()
            .map(|x| x.get_filename())
            .collect::<Vec<&String>>()
    );

    let paths = Arc::new(RwLock::new(Paths::default()));
    parse_rib_files(rib_files, &paths);

    // after parsing, take ownership if unique
    let paths: Paths = Arc::try_unwrap(paths)
        .expect("multiple Arc refs exist")
        .into_inner()
        .expect("RwLock poisoned");

    paths
}

/// Parse RIB files using multithreading
fn parse_rib_files(rib_files: &Vec<RibFile>, paths: &Arc<RwLock<Paths>>) {
    // Spin up a thread per file for parsing
    rib_files.into_par_iter().for_each(|rib_file| {
        let fp = rib_file.get_filename();
        info!("Parsing {}", fp);
        let peer_table = get_peer_table(fp);
        debug!("Peer Table for {}: {:#?}\n", fp, peer_table);

        let parser =
            BgpkitParser::new(fp.as_str()).unwrap_or_else(|_| panic!("Unable to parse {}", fp));

        if rib_files.len() == 1 {
            // Count of records excluding first entry (peer table)
            let total_records = parser.into_record_iter().skip(1).count();
            let num_threads = rayon::current_num_threads();
            // +1 to account for record count not being perfectly divisible by num_threads
            let per_thread_len = (total_records / num_threads) + 1;
            let parsed = Arc::new(AtomicU32::new(0));
            let mut thread_paths = (0..num_threads)
                .map(|_| Arc::new(RwLock::new(Paths::default())))
                .collect::<Vec<_>>();

            info!(
                "Going to parse {} records using {} threads, with {} records per thread",
                total_records, num_threads, per_thread_len
            );

            (0..num_threads).into_par_iter().for_each(|worker_id| {
                if per_thread_len == 0 {
                    return;
                }

                // Start from 1 to skip peer table
                let start = 1 + (worker_id * per_thread_len);
                let thread_parser = BgpkitParser::new(fp.as_str())
                    .unwrap_or_else(|_| panic!("Unable to parse {}", fp));
                let parsed = Arc::clone(&parsed);

                thread_parser
                    .into_record_iter()
                    .skip(start)
                    .take(per_thread_len)
                    .for_each(|mrt_entry| {
                        parse_mrt_entry(RecordData::new(
                            &mrt_entry,
                            &Arc::clone(&thread_paths[worker_id]),
                            &peer_table,
                            fp,
                        ));
                        let parsed = parsed.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
                        if parsed.is_multiple_of(100000) {
                            info!("Parsed {} records", parsed);
                        }
                    });
                info!("Thread {} finished parsing", worker_id);
            });

            info!(
                "Finished parsing {} records",
                parsed.load(std::sync::atomic::Ordering::SeqCst)
            );
            info!("Merging {} paths sets...", thread_paths.len());

            // Pairwise merge per-thread paths into the main paths.
            while thread_paths.len() > 1 {
                let merged_paths = thread_paths
                    .par_chunks_mut(2)
                    .map(|chunk| {
                        if chunk.len() == 2 {
                            let first = chunk.first().unwrap();
                            let second = chunk.get(1).unwrap();
                            first
                                .write()
                                .unwrap()
                                .merge_from(&mut second.write().unwrap());
                            info!("Merged two paths sets together");
                            Arc::clone(first)
                        } else if chunk.len() == 1 {
                            info!("Returning single paths set");
                            Arc::clone(chunk.first().unwrap())
                        } else {
                            panic!(
                                "Unexpected chunk length when merging paths: {}",
                                chunk.len()
                            );
                        }
                    })
                    .collect();
                thread_paths = merged_paths;
            }

            assert!(thread_paths.len() == 1);
            info!("Merging into global paths set...");
            paths
                .write()
                .unwrap()
                .merge_from(&mut thread_paths.pop().unwrap().write().unwrap());
            info!("Finished merging paths sets");
        } else {
            // If there are multiple files, just parse this file in this thread
            parser.into_record_iter().skip(1).for_each(|mrt_entry| {
                parse_mrt_entry(RecordData::new(
                    &mrt_entry,
                    &Arc::clone(paths),
                    &peer_table,
                    fp,
                ));
            });
        }

        info!("Parsed {}", fp,);
    });
}
