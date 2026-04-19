#[derive(Debug)]
pub struct RibFile {
    pub url: String,
    pub filename: String,
}

impl RibFile {
    pub fn new(url: String, filename: String) -> Self {
        if url.is_empty() && filename.is_empty() {
            panic!("URL and filename cannot be empty");
        }
        Self { url, filename }
    }

    pub fn get_filename(&self) -> &String {
        &self.filename
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let test_cases = vec![
            (
                "https://data.ris.ripe.net/rrc00/2024.01/bview.20240101.0000.gz",
                "bview.20240101.0000.gz",
            ),
            (
                "http://archive.routeviews.org/route-views.amsix/bgpdata/2024.01/RIBS/rib.20240101.0000.bz2",
                "rib.20240101.0000.bz2",
            ),
            ("file:///local/path/rib.mrt", "rib.mrt"),
        ];

        for (url, filename) in test_cases {
            let rib_file = RibFile::new(url.to_string(), filename.to_string());
            assert_eq!(rib_file.get_filename(), filename);
        }
    }

    #[test]
    fn test_new_empty_strings() {
        assert!(
            std::panic::catch_unwind(|| {
                RibFile::new(String::from(""), String::from(""));
            })
            .is_err()
        );
    }

    #[test]
    fn test_get_filename() {
        let filename = String::from("rib.20240101.0000.bz2");
        let rib_file = RibFile::new(
            String::from("http://example.com/rib.20240101.0000.bz2"),
            filename.clone(),
        );

        assert_eq!(rib_file.get_filename(), &filename);
    }
}
