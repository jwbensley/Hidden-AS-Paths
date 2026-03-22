use crate::clients::file::ensure_dir;
use crate::data::origin_as_paths::OriginAsPaths;
use crate::types::as_path::AsPath;
use crate::types::asn::Asn;
use crate::types::route::Route;
use core::panic;
use log::{debug, info};
use serde::Serialize;
use serde_json;
use std::collections::hash_map::{Keys, Values, ValuesMut};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::BufWriter;

/// Public API which provides access to all paths and routes.
/// Store all OriginAsPaths keyed by origin ASN.
#[derive(Debug, Serialize, Default)]
pub struct Paths {
    paths: HashMap<Asn, OriginAsPaths>,
}

impl PartialEq for Paths {
    fn eq(&self, other: &Self) -> bool {
        self.paths == other.paths
    }
}

impl Paths {
    pub fn new() -> Self {
        Paths {
            paths: HashMap::new(),
        }
    }

    fn has_origin_as_paths(&self, origin: &Asn) -> bool {
        debug!(
            "Existing paths for origin {}: {}",
            origin,
            self.paths.contains_key(origin)
        );
        self.paths.contains_key(origin)
    }

    fn get_origin_as_paths(&self, origin: &Asn) -> &OriginAsPaths {
        if self.has_origin_as_paths(origin) {
            self.paths.get(origin).unwrap()
        } else {
            panic!("No paths for origin {}", origin);
        }
    }

    pub fn has_route(&self, route: &Route) -> bool {
        debug!("Checking if route exists: {:#?}", route);
        let origin = route.get_origin();
        if !self.has_origin_as_paths(origin) {
            return false;
        };
        self.get_origin_as_paths(origin).has_route(route)
    }

    fn add_origin(&mut self, origin: Asn) {
        if self.has_origin_as_paths(&origin) {
            return;
        };
        self.paths
            .insert(origin.clone(), OriginAsPaths::new(origin, HashSet::new()));
    }

    fn get_origin_as_paths_mut(&mut self, origin: &Asn) -> &mut OriginAsPaths {
        if self.has_origin_as_paths(origin) {
            self.paths.get_mut(origin).unwrap()
        } else {
            panic!("No paths for origin {}", origin);
        }
    }

    pub fn add_origin_as_path(&mut self, as_path: AsPath) {
        if !self.has_origin_as_paths(as_path.get_origin()) {
            self.add_origin(as_path.get_origin().clone());
        }
        let origin_as_paths = self.get_origin_as_paths_mut(as_path.get_origin());
        origin_as_paths.add_as_path(as_path);
    }

    pub fn add_route(&mut self, route: Route, as_path: &AsPath) {
        debug!("Adding route {:#?}", route);
        if !self.has_route(&route) {
            self.add_origin(route.get_origin().clone());
            self.add_origin_as_path(as_path.clone());
            self.get_origin_as_paths_mut(route.get_origin())
                .add_route(route, as_path);
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

    fn remove_origin(&mut self, origin: &Asn) {
        if self.has_origin_as_paths(origin) {
            debug!("Removing AS paths for origin {}", origin);
            self.paths.remove(origin);
        } else {
            panic!(
                "Attempt to remove AS paths for non-existing origin {}",
                origin
            );
        }
    }

    pub fn remove_origins_with_one_or_less_as_paths(&mut self) {
        info!("Removing origins with one or less AS paths");

        let mut to_remove = Vec::new();
        for origin in self.get_origins() {
            if self.get_origin_as_paths(origin).len() <= 1 {
                to_remove.push(origin.to_owned());
            }
        }

        debug!(
            "Removing {} origins with one or less AS paths",
            to_remove.len(),
        );
        for origin in to_remove.iter() {
            self.remove_origin(origin);
        }
    }

    pub fn print_summary(&self) {
        info!(
            "Paths: {} origins, with {} AS paths",
            self.get_origins_count(),
            self.get_as_paths_count()
        );
    }

    pub fn remove_non_divergent_as_paths(&mut self) {
        info!("Removing non-divergent AS paths");
        for origin_as_paths in self.get_as_paths_mut() {
            origin_as_paths.remove_non_divergent_as_paths();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eq() {
        let mut paths_1 = Paths::new();
        paths_1.add_origin(Asn::get_mock(None));
        let mut paths_2 = Paths::new();
        paths_2.add_origin(Asn::get_mock(None));
        assert_eq!(paths_1, paths_2);

        paths_1.add_origin_as_path(AsPath::get_mock(None, None));
        assert_ne!(paths_1, paths_2);

        paths_2.add_origin_as_path(AsPath::get_mock(None, None));
        assert_eq!(paths_1, paths_2);

        paths_1.remove_origin(&Asn::get_mock(None));
        assert_ne!(paths_1, paths_2);
    }

    #[test]
    fn test_new() {
        let paths = Paths::new();
        assert_eq!(paths.get_origins_count(), 0);
        assert_eq!(paths.get_as_paths_count(), 0);
    }

    #[test]
    fn test_has_origin_as_paths() {
        let mut paths = Paths::new();
        paths.add_origin_as_path(AsPath::get_mock(None, None));

        assert!(paths.has_origin_as_paths(AsPath::get_mock(None, None).get_origin()));
        assert!(!paths.has_origin_as_paths(&Asn::get_mock(Some(10))));
    }

    #[test]
    fn test_get_origin_as_paths() {
        let mut paths = Paths::new();
        paths.add_origin_as_path(AsPath::get_mock(None, None));

        assert_eq!(
            paths.get_origin_as_paths(&Asn::get_mock(None)),
            &OriginAsPaths::get_mock(None, None)
        );
    }

    #[test]
    fn test_get_origin_as_paths_non_existing() {
        let paths = Paths::new();
        assert!(
            std::panic::catch_unwind(|| {
                paths.get_origin_as_paths(&Asn::get_mock(None));
            })
            .is_err()
        );
    }

    #[test]
    fn test_has_route() {
        let as_path = AsPath::get_mock(None, None);
        let mut paths = Paths::new();
        let route = Route::get_mock(None);
        assert!(!paths.has_route(&route));

        paths.add_route(route.clone(), &as_path);
        assert!(paths.has_route(&route));
    }

    #[test]
    fn test_add_origin() {
        let mut paths = Paths::new();
        assert_eq!(paths.get_as_paths().len(), 0);
        assert!(!paths.has_origin_as_paths(&Asn::get_mock(None)));

        paths.add_origin(Asn::get_mock(None));
        assert_eq!(paths.get_as_paths().len(), 1);
        assert!(paths.has_origin_as_paths(&Asn::get_mock(None)));
    }

    #[test]
    fn test_get_origin_as_paths_mut() {
        assert!(
            std::panic::catch_unwind(|| {
                let mut paths = Paths::new();
                paths.get_origin_as_paths_mut(&Asn::get_mock(None));
            })
            .is_err()
        );

        let mut paths = Paths::new();
        paths.add_origin_as_path(AsPath::get_mock(None, None));
        let origin_as_paths = paths.get_origin_as_paths_mut(&Asn::get_mock(None));

        assert_eq!(origin_as_paths.get_as_paths().len(), 1);
        assert_eq!(
            origin_as_paths.get_as_paths(),
            &HashSet::from([AsPath::get_mock(None, None)])
        );

        origin_as_paths.add_as_path(AsPath::get_mock(
            Some(Vec::from([Asn::get_mock(Some(10))])),
            None,
        ));
        assert_eq!(origin_as_paths.get_as_paths().len(), 2);
        assert_eq!(
            origin_as_paths.get_as_paths(),
            &HashSet::from([
                AsPath::get_mock(None, None),
                AsPath::get_mock(Some(Vec::from([Asn::get_mock(Some(10))])), None)
            ])
        );
    }

    #[test]
    fn test_add_origin_as_path() {
        let mut paths = Paths::new();
        assert_eq!(paths.get_as_paths_count(), 0);

        paths.add_origin_as_path(AsPath::get_mock(None, None));
        assert_eq!(paths.get_as_paths_count(), 1);
        assert!(paths.has_origin_as_paths(&Asn::get_mock(None)));

        paths.add_origin_as_path(AsPath::get_mock(
            Some(Vec::from([Asn::get_mock(Some(10))])),
            None,
        ));
        assert_eq!(paths.get_as_paths_count(), 2);
    }

    #[test]
    fn test_add_route() {
        let as_path_1 = AsPath::get_mock(Some(vec![Asn::new(2), Asn::new(1)]), None);
        let as_path_2 = AsPath::get_mock(Some(vec![Asn::new(3), Asn::new(1)]), None);
        let mut paths = Paths::new();
        let route = Route::get_mock(None);

        assert_eq!(paths.get_origins_count(), 0);
        paths.add_route(route.clone(), &as_path_1);
        assert_eq!(paths.get_origins_count(), 1);
        assert!(paths.has_route(&route));

        // Adding same route again should not increase origin count
        // even via different AS path
        paths.add_route(route.clone(), &as_path_2);
        assert_eq!(paths.get_origins_count(), 1);
    }

    #[test]
    fn test_get_origins_count() {
        let mut paths = Paths::new();
        assert_eq!(paths.get_origins_count(), 0);

        paths.add_origin(Asn::get_mock(None));
        assert_eq!(paths.get_origins_count(), 1);

        paths.add_origin(Asn::get_mock(Some(10)));
        assert_eq!(paths.get_origins_count(), 2);
    }

    #[test]
    fn test_get_as_paths() {
        let mut paths = Paths::new();
        assert_eq!(paths.get_as_paths().len(), 0);

        paths.add_origin_as_path(AsPath::get_mock(None, None));
        assert_eq!(paths.get_as_paths().len(), 1);
    }

    #[test]
    fn test_get_as_paths_count() {
        let mut paths = Paths::new();
        assert_eq!(paths.get_as_paths_count(), 0);

        paths.add_origin_as_path(AsPath::get_mock(None, None));
        assert_eq!(paths.get_as_paths_count(), 1);

        paths.add_origin_as_path(AsPath::get_mock(
            Some(Vec::from([Asn::get_mock(Some(10))])),
            None,
        ));
        assert_eq!(paths.get_as_paths_count(), 2);
    }

    #[test]
    fn test_get_as_paths_mut() {
        let mut paths = Paths::new();
        paths.add_origin_as_path(AsPath::get_mock(None, None));
        assert_eq!(paths.get_as_paths_count(), 1);

        let mut as_paths_mut = paths.get_as_paths_mut();
        assert_eq!(as_paths_mut.len(), 1);
        as_paths_mut
            .next()
            .unwrap()
            .add_as_path(AsPath::get_mock(Some(Vec::from([Asn::new(10)])), None));

        assert_eq!(paths.get_as_paths_count(), 2);
    }

    #[test]
    fn test_get_origins() {
        let mut paths = Paths::new();
        assert_eq!(paths.get_origins().len(), 0);

        paths.add_origin(Asn::get_mock(None));
        assert_eq!(paths.get_origins().len(), 1);

        paths.add_origin(Asn::get_mock(Some(10)));
        assert_eq!(paths.get_origins().len(), 2);
    }

    #[test]
    fn test_remove_origin() {
        let mut paths = Paths::new();
        paths.add_origin(Asn::get_mock(None));
        assert_eq!(paths.get_origins_count(), 1);

        paths.remove_origin(&Asn::get_mock(None));
        assert_eq!(paths.get_origins_count(), 0);
    }

    #[test]
    fn test_remove_origin_non_existing() {
        let mut paths = Paths::new();
        assert!(
            std::panic::catch_unwind(move || {
                paths.remove_origin(&Asn::get_mock(None));
            })
            .is_err()
        );
    }

    #[test]
    fn test_remove_origins_with_one_or_less_as_paths() {
        let mut paths = Paths::new();
        paths.add_origin_as_path(AsPath::get_mock(Some(Vec::from([Asn::new(10)])), None));
        paths.add_origin_as_path(AsPath::get_mock(
            Some(Vec::from([Asn::new(20), Asn::new(10)])),
            None,
        ));
        paths.add_origin_as_path(AsPath::get_mock(Some(Vec::from([Asn::new(30)])), None));

        assert_eq!(paths.get_origins_count(), 2);

        paths.remove_origins_with_one_or_less_as_paths();
        assert_eq!(paths.get_origins_count(), 1);
    }

    #[test]
    fn test_remove_single_hop_as_paths_empty() {
        let mut paths = Paths::new();
        assert_eq!(paths.get_as_paths_count(), 0);

        paths.remove_single_hop_as_paths();
        assert_eq!(paths.get_as_paths_count(), 0);
    }

    #[test]
    fn test_remove_single_hop_as_paths_only_single_hop() {
        let mut paths = Paths::new();
        paths.add_origin_as_path(AsPath::new(Vec::from([Asn::new(10)]), Vec::new()));
        paths.add_origin_as_path(AsPath::new(Vec::from([Asn::new(20)]), Vec::new()));
        assert_eq!(paths.get_as_paths_count(), 2);

        paths.remove_single_hop_as_paths();
        assert_eq!(paths.get_as_paths_count(), 0);
    }

    #[test]
    fn test_remove_single_hop_as_paths_only_multi_hop() {
        let mut paths = Paths::new();
        paths.add_origin_as_path(AsPath::new(
            Vec::from([Asn::new(20), Asn::new(10)]),
            Vec::new(),
        ));
        paths.add_origin_as_path(AsPath::new(
            Vec::from([Asn::new(30), Asn::new(20), Asn::new(10)]),
            Vec::new(),
        ));
        assert_eq!(paths.get_as_paths_count(), 2);

        paths.remove_single_hop_as_paths();
        assert_eq!(paths.get_as_paths_count(), 2);
    }

    #[test]
    fn test_remove_single_hop_as_paths_mixed() {
        let mut paths = Paths::new();
        paths.add_origin_as_path(AsPath::new(Vec::from([Asn::new(10)]), Vec::new()));
        paths.add_origin_as_path(AsPath::new(
            Vec::from([Asn::new(20), Asn::new(10)]),
            Vec::new(),
        ));
        paths.add_origin_as_path(AsPath::new(
            Vec::from([Asn::new(30), Asn::new(10)]),
            Vec::new(),
        ));
        paths.add_origin_as_path(AsPath::new(Vec::from([Asn::new(50)]), Vec::new()));

        assert_eq!(paths.get_as_paths_count(), 4);
        assert_eq!(paths.get_origins_count(), 2);

        paths.remove_single_hop_as_paths();
        assert_eq!(paths.get_as_paths_count(), 2);
        assert_eq!(paths.get_origins_count(), 2);
        assert!(paths.has_origin_as_paths(&Asn::new(10)));
        assert!(paths.has_origin_as_paths(&Asn::new(50)));
    }
}
