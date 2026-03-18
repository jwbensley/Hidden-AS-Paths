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
