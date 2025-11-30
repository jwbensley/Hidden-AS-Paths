// static THREADS: u32 = 2;

pub mod asp_tree;
pub mod http;
pub mod parse;
pub mod ribs;
use std::io::Write;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format(|buf, record| {
            let ts = buf.timestamp_micros();
            writeln!(
                buf,
                "{}: {:?}: {}: {}",
                ts,
                std::thread::current().id(),
                buf.default_level_style(record.level()),
                record.args()
            )
        })
        .init();

    let date = "2025-09-22";
    let dir = "./mrts";
    let rib_files = ribs::rib_getter::download_ribs_for_day(date, dir);
    parse::rib_parser::parse_ribs(dir, &rib_files);
}
