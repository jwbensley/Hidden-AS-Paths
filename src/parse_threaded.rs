use crate::args::cli_args::CliArgs;
use crate::data::paths::Paths;
use crate::mrt_data::MrtData;
use crate::parse_mrt::{get_peer_table, parse_mrt_entry};
use crate::ribs::rib_getter::RibFile;
use bgpkit_parser::BgpkitParser;
use log::{debug, info};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rayon::prelude::*;
use std::sync::{Arc, RwLock};

/// Setup and call parallel parsing of RIB files
pub fn init_parallel_parsing(rib_files: &Vec<RibFile>, args: &CliArgs) -> Paths {
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
    let mut paths: Paths = Arc::try_unwrap(paths)
        .expect("multiple Arc refs exist")
        .into_inner()
        .expect("RwLock poisoned");

    info!(
        "Found {} origins with {} AS paths.",
        paths.get_origins_count(),
        paths.get_as_paths_count(),
    );

    debug! {"{:#?}", paths};
    paths.remove_single_hop_as_paths();
    paths.remove_origins_with_single_as_path();
    paths.to_file(&args.paths);

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
            // If there is only one file, parse that file across all available threads
            parser
                .into_record_iter()
                .skip(1)
                .par_bridge()
                .for_each(|mrt_entry| {
                    parse_mrt_entry(MrtData::new(
                        &mrt_entry,
                        &Arc::clone(&paths),
                        &peer_table,
                        fp,
                    ))
                });
        } else {
            // If there are multiple files, just parse this file in this thread
            parser.into_record_iter().skip(1).for_each(|mrt_entry| {
                parse_mrt_entry(MrtData::new(
                    &mrt_entry,
                    &Arc::clone(&paths),
                    &peer_table,
                    fp,
                ))
            });
        }

        info!("Parsed {}", fp,);
    });
}
