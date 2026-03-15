pub mod rib_getter {
    use crate::file::ensure_dir;
    use crate::http::http_client::download_file;
    use bgpkit_broker::BgpkitBroker;
    use log::{debug, info};
    use rayon::iter::{IntoParallelIterator, ParallelIterator};
    use std::path::Path;

    #[derive(Debug)]
    pub struct RibFile {
        pub url: String,
        pub filename: String,
    }

    impl RibFile {
        pub fn new(url: String, filename: String) -> Self {
            Self { url, filename }
        }
        pub fn get_filename(&self) -> &String {
            &self.filename
        }
    }

    /// Download all the ribs files for a specific day
    pub fn download_ribs_for_day(date: &str, dir: &str) -> Vec<RibFile> {
        info!("Downloading MRT RIBs for {}", date);
        let rib_files = get_rib_list_for_day(date, dir);
        download_ribs_to_dir(&rib_files);
        rib_files
    }

    fn download_ribs_to_dir(rib_files: &Vec<RibFile>) {
        ensure_dir(rib_files[0].get_filename());
        rib_files
            .into_par_iter()
            .for_each(|rib_file| download_file(&rib_file.url, Path::new(&rib_file.filename)));
    }

    /// Return a list of available RIBs for a specific day (with details like download URL)
    fn get_rib_list_for_day(date: &str, dir: &str) -> Vec<RibFile> {
        let broker = BgpkitBroker::new().ts_start(date).ts_end(date);
        let ribs = broker.daily_ribs().unwrap();
        debug!("Found {} MRT files for date {}", ribs.len(), date);

        let mut rib_files = Vec::<RibFile>::new();
        for rib in ribs {
            let basename = Path::new(&rib.url).file_name().unwrap().to_str().unwrap();

            let source = if rib.collector_id.starts_with("rrc") {
                String::from("ris")
            } else {
                String::from("route-views")
            };

            let filename = if rib.collector_id.starts_with(&source) {
                format!("{}/{}.{}", dir, rib.collector_id, basename)
            } else {
                format!("{}/{}.{}.{}", dir, &source, rib.collector_id, basename)
            };

            rib_files.push(RibFile {
                url: rib.url,
                filename,
            });
        }

        rib_files
    }
}
