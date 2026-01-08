// #![warn(missing_docs)]

#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

use crate::ribs::rib_getter::RibFile;

pub mod args;
pub mod http;
pub mod logging;
pub mod mrt_as_path;
pub mod mrt_origin_as_paths;
pub mod mrt_paths;
pub mod mrt_route;
pub mod parse;
pub mod ribs;

fn main() {
    let args = args::cli_args::parse_cli_arg();
    if args.debug {
        logging::setup_loggin("debug");
    } else {
        logging::setup_loggin("info");
    }

    let rib_files: Vec<RibFile> = if args.download() {
        ribs::rib_getter::download_ribs_for_day(args.get_ribs_ymd(), args.get_ribs_path())
    } else {
        args.get_rib_files()
            .iter()
            .map(|filename| RibFile {
                url: String::new(),
                filename: filename.clone(),
            })
            .collect()
    };

    let path_data = parse::rib_parser::get_path_data(&rib_files, &args.threads);
    println!(
        "Remaining data is for {} origins with {} AS paths",
        path_data.count_origins(),
        path_data.count_as_paths()
    );
}
