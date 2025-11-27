pub mod http {
    use log::{debug, info};
    use reqwest::blocking;
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;

    pub fn download_file(url: &str, dest: &Path) {
        if dest.exists() {
            debug!(
                "Not GETting URL {}, output file already exists {}",
                url,
                dest.to_str().unwrap(),
            );
            return;
        }

        debug!("GET'ing URL {}", url);

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
}
