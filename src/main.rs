pub mod as_paths;
pub mod logging;
pub mod http;
pub mod parse;
pub mod path_data;
pub mod ribs;


fn main() {
    logging::setup_loggin();
    let date = "2025-09-22";
    let dir = "./mrts";
    let rib_files = ribs::rib_getter::download_ribs_for_day(date, dir);
    let path_data = parse::rib_parser::get_path_data(dir, &rib_files);
}
