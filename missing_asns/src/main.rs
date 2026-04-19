use crate::filter::path_filters::filter_results;
use crate::parse_threaded::get_paths;

pub mod args;
pub mod clients;
pub mod data;
pub mod filter;
pub mod logging;
pub mod parse_mrt;
pub mod parse_threaded;
pub mod types;

fn main() {
    let args = args::cli_args::parse_cli_arg();
    if args.debug {
        logging::setup_logging("debug");
    } else {
        logging::setup_logging("info");
    }

    let mut paths = get_paths(&args);
    filter_results(&mut paths, &args);
}
