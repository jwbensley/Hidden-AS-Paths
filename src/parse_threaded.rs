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
use rayon::prelude::*;
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
            // Defined a thread safe atomic counter.
            // All threads will increment this counter when they finish parsing a record.
            // If the counter is a multiple of 100000, print the number of records parsed so far.
            let parsed = Arc::new(AtomicU32::new(0));

            // If there is only one file, parse that file across all available threads
            parser
                .into_record_iter()
                .skip(1)
                .par_bridge()
                .for_each(|mrt_entry| {
                    parse_mrt_entry(RecordData::new(
                        &mrt_entry,
                        &Arc::clone(paths),
                        &peer_table,
                        fp,
                    ));
                    let parsed = parsed.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
                    if parsed.is_multiple_of(100000) {
                        info!("Parsed {} records", parsed);
                    }
                });
            info!(
                "Finished parsing {} records",
                parsed.load(std::sync::atomic::Ordering::SeqCst)
            );
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
