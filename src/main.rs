pub mod args;
pub mod clients;
pub mod data;
pub mod logging;
pub mod parse_mrt;
pub mod parse_threaded;
pub mod types;

use crate::args::cli_args::CliArgs;
use crate::args::cli_args::RibsSource;
use crate::clients::mrt_archives::download_ribs_for_day;
use crate::clients::peeringdb::get_ixp_rs_asns;
use crate::data::paths::Paths;
use crate::parse_threaded::init_parallel_parsing;
use crate::types::rib::RibFile;
use rayon::ThreadPoolBuilder;

fn get_paths(args: &CliArgs) -> Paths {
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

fn filter_paths(mut paths: Paths, args: &CliArgs) {
    paths.print_summary();
    paths.remove_single_hop_as_paths();
    paths.print_summary();
    paths.remove_origins_with_one_or_less_as_paths();
    paths.print_summary();
    paths.remove_origins_with_non_divergent_paths();
    paths.print_summary();
    paths.to_file(&args.paths);
}

fn filter_paths_by_community() {
    // ASN 0 is a special ASN used by many networks as an action community.
    // asns.insert(0, 0);
}

fn main() {
    let args = args::cli_args::parse_cli_arg();
    if args.debug {
        logging::setup_logging("debug");
    } else {
        logging::setup_logging("info");
    }

    ThreadPoolBuilder::new()
        .num_threads((args.threads).try_into().unwrap())
        .build_global()
        .unwrap();

    let _ixp_rs_asns = get_ixp_rs_asns(&args.peeringdb);
    let paths = get_paths(&args);
    filter_paths(paths, &args);
}
