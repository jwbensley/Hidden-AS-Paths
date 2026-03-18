use crate::clients::file::ensure_dir;
use crate::data::origin_as_paths::OriginAsPaths;
use crate::types::as_path::AsPath;
use crate::types::asn::Asn;
use crate::types::route::Route;
use bgpkit_parser::models::Asn as BgpKitAsn;
use core::panic;
use log::{debug, info};
use serde::Serialize;
use serde_json;
use std::collections::HashMap;
use std::collections::hash_map::{Keys, Values, ValuesMut};
use std::fs::File;
use std::io::BufWriter;

/// Public API which provides access to all paths and routes.
/// Store all OriginAsPaths keyed by origin ASN.
#[derive(Debug, Serialize, Default)]
pub struct Paths {
    paths: HashMap<Asn, OriginAsPaths>,
}

// impl Default for Paths {
//     fn default() -> Self {
//         Self::new()
//     }
// }

impl PartialEq for Paths {
    fn eq(&self, other: &Self) -> bool {
        self.paths == other.paths
    }
}

impl Paths {
    fn has_as_paths_for_origin(&self, origin: &Asn) -> bool {
        debug!(
            "Existing paths for origin {}: {}",
            origin,
            self.paths.contains_key(origin)
        );
        self.paths.contains_key(origin)
    }

    fn get_as_paths_for_origin(&self, origin: &Asn) -> &OriginAsPaths {
        if self.has_as_paths_for_origin(origin) {
            self.paths.get(origin).unwrap()
        } else {
            panic!("No paths for origin {}", origin);
        }
    }

    pub fn has_route(&self, route: &Route) -> bool {
        let origin = route.get_origin();
        if !self.has_as_paths_for_origin(origin) {
            return false;
        };
        self.get_as_paths_for_origin(origin).has_route(route)
    }

    fn add_origin(&mut self, origin: Asn) {
        if self.has_as_paths_for_origin(&origin) {
            return;
        };
        self.paths
            .insert(origin.clone(), OriginAsPaths::new(origin, Vec::new()));
    }

    fn get_as_paths_for_origin_mut(&mut self, origin: &Asn) -> &mut OriginAsPaths {
        if self.has_as_paths_for_origin(origin) {
            self.paths.get_mut(origin).unwrap()
        } else {
            panic!("No paths for origin {}", origin);
        }
    }

    fn add_as_path(&mut self, as_path: AsPath) {
        let origin_as_paths = self.get_as_paths_for_origin_mut(as_path.get_origin());
        origin_as_paths.add_as_path(as_path);
    }

    fn add_route(&mut self, route: Route) {
        self.get_as_paths_for_origin_mut(route.get_origin())
            .add_route(route);
    }

    pub fn insert_route(&mut self, route: Route) {
        debug!("Adding route {:#?}", route);
        if !self.has_route(&route) {
            self.add_origin(route.get_origin().clone());
            self.add_as_path(route.get_as_path().clone());
            self.add_route(route);
        }
    }

    pub fn get_origins_count(&self) -> usize {
        self.paths.len()
    }

    fn get_as_paths(&self) -> Values<'_, Asn, OriginAsPaths> {
        self.paths.values()
    }

    pub fn get_as_paths_count(&self) -> usize {
        let mut total = 0;
        for origin_as_paths in self.get_as_paths() {
            total += origin_as_paths.len();
        }
        total
    }

    pub fn to_file(&self, filename: &String) {
        ensure_dir(filename);
        let writer = BufWriter::new(File::create(filename).unwrap());
        serde_json::to_writer_pretty(writer, self).unwrap();
        info!("Wrote JSON to {}", filename);
    }

    fn get_as_paths_mut(&mut self) -> ValuesMut<'_, Asn, OriginAsPaths> {
        self.paths.values_mut()
    }

    /// Remove AS Paths which only have a single ASN in the path
    pub fn remove_single_hop_as_paths(&mut self) {
        info!("Removing single-hop AS paths");

        for origin_as_paths in self.get_as_paths_mut() {
            origin_as_paths.remove_single_hop_paths();
        }
    }

    fn get_origins(&self) -> Keys<'_, Asn, OriginAsPaths> {
        self.paths.keys()
    }

    fn remove_as_paths_for_origin(&mut self, origin: &Asn) {
        if self.has_as_paths_for_origin(origin) {
            debug!("Removing AS paths for origin {}", origin);
            self.paths.remove(origin);
        } else {
            panic!(
                "Attempt to remove AS paths for non-existing origin {}",
                origin
            );
        }
    }

    /// Remove origins which only have a single AS path
    pub fn remove_origins_with_single_as_path(&mut self) {
        info!("Removing origins with only one AS path");

        let mut to_remove = Vec::new();
        for origin in self.get_origins() {
            if self.get_as_paths_for_origin(origin).len() == 1 {
                to_remove.push(origin.to_owned());
            }
        }

        debug!("Removing {} origins with single AS path", to_remove.len(),);
        for origin in to_remove.iter() {
            self.remove_as_paths_for_origin(origin);
        }
    }

    pub fn print_summary(&self) {
        info!(
            "Summary: {} origins, with {} AS paths",
            self.get_origins_count(),
            self.get_as_paths_count()
        );
    }

    // pub fn find_origins_with_divergent_paths(&self) {
    //     info!("Searching for divergent paths");
    //     for origin_as_paths in self.get_as_paths() {
    //         let divergent_paths = origin_as_paths.find_divergent_paths();
    //         println!("{:#?}", divergent_paths);
    //         if true {
    //             break;
    //         }
    //     }
    // }

    // pub fn pop_as_paths_for_origin(&mut self, origin: &Asn) -> OriginAsPaths {
    //     self.paths.remove(origin).unwrap()
    // }
}
