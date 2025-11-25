use log::info;
// static THREADS: u32 = 2;

pub mod http;
pub mod parse;
pub mod ribs;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let date = "2025-09-22";
    let dir = "./mrts";
    let rib_files = ribs::ribs::download_ribs_for_day(date, dir);
    parse::parse::parse_ribs(dir, &rib_files);
}
