pub mod args;
pub mod data;
pub mod file;
pub mod http;
pub mod logging;
pub mod mrt_data;
pub mod mrt_types;
pub mod parse_mrt;
pub mod parse_threaded;
pub mod ribs;

use crate::parse_threaded::init_parallel_parsing;
use crate::ribs::rib_getter::download_ribs_for_day;
use crate::{args::cli_args::RibsSource, ribs::rib_getter::RibFile};
use rayon::ThreadPoolBuilder;

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

    let mut paths = match args.ribs_source {
        // Download MRT files and then parse them using one thread per file
        RibsSource::Download(_) => {
            let rib_files = download_ribs_for_day(args.get_ribs_ymd(), args.get_ribs_path());
            init_parallel_parsing(&rib_files, &args)
        }

        // Parse a single existing file split across multiple threads
        RibsSource::File(_) => {
            let rib_files = Vec::from([RibFile::new(String::new(), args.get_rib_file().clone())]);
            init_parallel_parsing(&rib_files, &args)
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

            init_parallel_parsing(&rib_files, &args)
        }
    };
}
