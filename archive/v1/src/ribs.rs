pub mod rib_getter {
    use crate::http::http;
    use bgpkit_broker::BgpkitBroker;
    use log::{debug, info};
    use std::fs;
    use std::path::Path;

    #[derive(Debug)]
    pub struct RibFile {
        pub url: String,
        pub filename: String,
    }

    pub fn download_ribs_for_day(date: &str, dir: &str) -> Vec<RibFile> {
        /*
         * Download all the ribs files for a specific day
         */
        let rib_files = get_rib_list_for_day(date);
        download_ribs_to_dir(dir, &rib_files);
        rib_files
    }

    fn download_ribs_to_dir(dir: &str, rib_files: &Vec<RibFile>) {
        let mrt_path = Path::new(dir);
        if !mrt_path.exists() {
            debug!("Creating path: {}", mrt_path.to_str().unwrap());
            fs::create_dir(mrt_path).unwrap();
        }

        for rib_file in rib_files {
            let dest = mrt_path.join(&rib_file.filename);
            http::download_file(&rib_file.url, &dest)
        }
    }

    fn get_rib_list_for_day(date: &str) -> Vec<RibFile> {
        /*
         * Return a list of RIBs for a specific day
         */

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
                filename,
            });
        }

        rib_files
    }
}
