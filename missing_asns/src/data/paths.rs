use crate::clients::file::ensure_dir;
use crate::types::as_path::AsPath;
use crate::types::route::Route;
use crate::types::sorted_hashmap::SortedHashMap;
use core::panic;
use log::{debug, info};
use serde::ser::SerializeStruct as _;
use serde::{Serialize, Serializer};
use serde_json;
use serde_json::to_string_pretty;
use std::collections::HashMap;
use std::collections::hash_map::Drain;
use std::fs::File;
use std::io::{BufWriter, Write};

/// Public API which provides access to all paths and routes.
/// Store unique AS paths, an one example route per path.
#[derive(Debug, Default)]
pub struct Paths {
    paths: HashMap<AsPath, Route>,
}

impl PartialEq for Paths {
    fn eq(&self, other: &Self) -> bool {
        self.paths == other.paths
    }
}

impl Serialize for Paths {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize the paths map sorted by the keys (origin ASNs) for deterministic output
        let sorted_paths = SortedHashMap(&self.paths);
        let mut state = serializer.serialize_struct("Paths", 1)?;
        state.serialize_field("paths", &sorted_paths)?;
        state.end()
    }
}

impl Paths {
    pub fn new() -> Self {
        Paths {
            paths: HashMap::new(),
        }
    }

    pub fn has_as_path(&self, as_path: &AsPath) -> bool {
        debug!(
            "Existing AS path {:?}: {}",
            as_path,
            self.paths.contains_key(as_path)
        );
        self.paths.contains_key(as_path)
    }

    pub fn add(&mut self, as_path: AsPath, route: Route) {
        if self.has_as_path(&as_path) {
            return;
        };
        self.paths.insert(as_path, route);
    }

    pub fn to_file(&self, filename: &String) {
        ensure_dir(filename);
        let mut writer = BufWriter::new(File::create(filename).unwrap());
        let json = to_string_pretty(&self).unwrap();
        writer
            .write_all(json.as_bytes())
            .unwrap_or_else(|_| panic!("Unable to write to file {}", filename));
        info!("Wrote JSON to {}", filename);
    }

    fn pop_all(&mut self) -> Drain<'_, AsPath, Route> {
        self.paths.drain()
    }

    pub fn merge_from(&mut self, other: &mut Paths) {
        // Drain other's set so this map take ownership
        for (as_path, route) in other.pop_all() {
            if !self.has_as_path(&as_path) {
                self.add(as_path, route);
            }
        }
    }
}
