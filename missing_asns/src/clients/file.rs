use log::debug;
use std::{fs, path::Path};

/// Ensure the path to a file exists
pub fn ensure_dir(filename: &String) {
    let path = Path::new(filename).parent().unwrap();
    if !path.exists() {
        debug!("Creating dir: {}", path.to_str().unwrap());
        fs::create_dir(path).unwrap();
    }
}
