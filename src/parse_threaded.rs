pub mod threaded_parser {
    use crate::args::cli_args::CliArgs;
    use crate::parse_mrt::mrt_parser::{MrtData, get_peer_id_map, parse_mrt_entry};
    use crate::ribs::rib_getter::RibFile;
    use bgpkit_parser::BgpkitParser;
    use log::{debug, info};
    use rayon::iter::{IntoParallelIterator, ParallelIterator};
    use rayon::prelude::*;
    use std::sync::{Arc, RwLock};

    /// Setup and call parallel parsing of RIB files
    pub fn init_parallel_parsing(rib_files: &Vec<RibFile>, args: &CliArgs) {
        info!("Going to parse {} RIB files", rib_files.len());
        debug!(
            "{:?}",
            rib_files
                .iter()
                .map(|x| x.get_filename())
                .collect::<Vec<&String>>()
        );

        let peering_data = Arc::new(RwLock::new(PeeringData::default()));

        parse_rib_files(rib_files, &asn_mappings, &peering_data, &triple_t1_paths);

        debug! {"{:#?}", peering_data.read().unwrap()};
        peering_data.read().unwrap().to_file(&args.peering_data);
    }

    /// Parse RIB files using multithreading
    fn parse_rib_files(rib_files: &Vec<RibFile>, peering_data: &Arc<RwLock<PeeringData>>) {
        // Spin up a thread per file for parsing
        rib_files.into_par_iter().for_each(|rib_file| {
            let fp = rib_file.get_filename();
            info!("Parsing {}", fp);
            let peer_id_map = get_peer_id_map(fp);
            debug!("Peer Map for {}: {:#?}\n", fp, peer_id_map);

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
                            &Arc::clone(&peering_data),
                            &peer_id_map,
                            fp,
                        ))
                    });
            } else {
                // If there are multiple files, just parse this file in this thread
                parser.into_record_iter().skip(1).for_each(|mrt_entry| {
                    parse_mrt_entry(MrtData::new(
                        &mrt_entry,
                        &Arc::clone(&peering_data),
                        &peer_id_map,
                        fp,
                    ))
                });
            }

            info!("Parsed {}", fp,);
        });
    }
}
