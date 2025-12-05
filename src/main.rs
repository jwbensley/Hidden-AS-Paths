// static THREADS: u32 = 2;

pub mod asp_tree;
pub mod http;
pub mod parse;
pub mod ribs;
use log::info;
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

    let all_as_sequences = parse::rib_parser::parse_ribs(dir, &rib_files);
    let mut as_sequences = asp_tree::asp_trees::merge_sequences(all_as_sequences);
    as_sequences.print_total();
    as_sequences.remove_single_paths();
    as_sequences.print_total();
    as_sequences.print_as_paths();
}
