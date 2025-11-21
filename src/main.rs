use bgpkit_broker::BgpkitBroker;
// use bgpkit_parser::BgpkitParser;
use log::info;
use reqwest::blocking;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

// static THREADS: u32 = 2;

#[derive(Debug)]
struct RibFile {
    url: String,
    filename: String,
}

fn get_rib_files(date: &str) -> Vec<RibFile> {
    let broker = BgpkitBroker::new().ts_start(date).ts_end(date);
    let ribs = broker.daily_ribs().unwrap();
    info!("Found {} MRT files for date {}", ribs.len(), date,);

    let mut rib_files = Vec::<RibFile>::new();
    for rib in ribs {
        let basename = Path::new(&rib.url).file_name().unwrap().to_str().unwrap();

        let source = if rib.collector_id.starts_with("rrc") {
            String::from("ris")
        } else {
            String::from("route-views")
        };

        let filename = if rib.collector_id.starts_with(&source) {
            format!("{}.{}", rib.collector_id, basename)
        } else {
            format!("{}.{}.{}", &source, rib.collector_id, basename)
        };

        rib_files.push(RibFile {
            url: rib.url,
            filename: filename,
        });
    }

    return rib_files;
}

fn download_file(url: &str, dest: &Path) {
    if dest.exists() {
        info!(
            "Not GETting URL {}, output file already exists {}",
            url,
            dest.to_str().unwrap(),
        );
        return;
    }

    info!("GET'ing URL {}", url);

    let response = blocking::get(url)
        .map_err(|e| format!("HTTP GET failed: {}", e))
        .unwrap();
    let content = response
        .bytes()
        .map_err(|e| format!("Failed to read response bytes: {}", e))
        .unwrap();

    File::create(&dest)
        .map_err(|e| format!("Failed to create file: {}", e))
        .unwrap()
        .write_all(&content)
        .unwrap();

    info!("Wrote to file {}", dest.to_str().unwrap());
}

fn download_mrt_files(rib_files: Vec<RibFile>) {
    let mrt_path = Path::new("./mrts");
    if !mrt_path.exists() {
        info!("Creating path: {}", mrt_path.to_str().unwrap());
        fs::create_dir(mrt_path).unwrap();
    }

    for rib_file in rib_files {
        let dest = mrt_path.join(rib_file.filename);
        download_file(&rib_file.url, &dest)
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let date = "2025-09-22";
    let rib_files = get_rib_files(date);

    download_mrt_files(rib_files);
    // let parser = BgpkitParser::new(
    //     "http://archive.routeviews.org/route-views.bdix/bgpdata/2025.09/RIBS/rib.20250922.0000.bz2",
    // )
    // .unwrap();
}
