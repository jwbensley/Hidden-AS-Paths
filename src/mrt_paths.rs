pub mod path_data {
    use crate::mrt_as_path::as_path::AsPath;
    use crate::mrt_origin_as_paths::origin_as_paths::OriginAsPaths;
    use crate::mrt_route::route::Route;
    use bgpkit_parser::models::Asn;
    use core::panic;
    use log::{debug, info};
    use std::collections::HashMap;
    use std::collections::hash_map::{Keys, Values};

    /// Public API which provides access to all paths and routes.
    /// Store all AsPathCollections keyed by origin ASN.
    #[derive(Debug)]
    pub struct PathData {
        as_paths: HashMap<Asn, OriginAsPaths>,
    }

    impl Default for PathData {
        fn default() -> Self {
            Self::new()
        }
    }

    impl PartialEq for PathData {
        fn eq(&self, other: &Self) -> bool {
            self.as_paths == other.as_paths
        }
    }

    impl PathData {
        pub fn new() -> Self {
            PathData {
                as_paths: HashMap::<Asn, OriginAsPaths>::new(),
            }
        }

        fn add_as_path(&mut self, as_path: Vec<Asn>) {
            let origin_as_paths = self.get_as_paths_for_origin_mut(as_path.last().unwrap());
            let asp = AsPath::new(as_path);
            origin_as_paths.add_as_path(asp);
        }

        fn add_origin(&mut self, origin: Asn) {
            if self.has_as_paths_for_origin(&origin) {
                return;
            };
            self.as_paths.insert(origin, OriginAsPaths::new(origin));
        }

        fn add_origin_as_paths(&mut self, origin: &Asn, origin_as_paths: &OriginAsPaths) {
            if !self.has_as_paths_for_origin(origin) {
                self.add_origin(*origin);
            }
            self.get_as_paths_for_origin_mut(origin)
                .merge_from(origin_as_paths);
        }

        fn add_route(&mut self, route: Route) {
            self.get_as_paths_for_origin_mut(route.get_origin())
                .add_route(route);
        }

        pub fn count_as_paths(&self) -> usize {
            let mut total = 0;
            for origin_as_paths in self.get_as_paths() {
                total += origin_as_paths.len();
            }
            total
        }

        pub fn count_origins(&self) -> usize {
            self.as_paths.len()
        }

        fn get_as_paths(&self) -> Values<'_, Asn, OriginAsPaths> {
            self.as_paths.values()
        }

        fn get_as_paths_for_origin(&self, origin: &Asn) -> &OriginAsPaths {
            if self.has_as_paths_for_origin(origin) {
                self.as_paths.get(origin).unwrap()
            } else {
                panic!("No paths for origin {}", origin);
            }
        }

        fn get_as_paths_for_origin_mut(&mut self, origin: &Asn) -> &mut OriginAsPaths {
            if self.has_as_paths_for_origin(origin) {
                self.as_paths.get_mut(origin).unwrap()
            } else {
                panic!("No paths for origin {}", origin);
            }
        }

        fn get_origins(&self) -> Keys<'_, Asn, OriginAsPaths> {
            self.as_paths.keys()
        }

        fn has_as_paths_for_origin(&self, origin: &Asn) -> bool {
            debug!(
                "Existing paths for origin {}: {}",
                origin,
                self.as_paths.contains_key(origin)
            );
            self.as_paths.contains_key(origin)
        }

        fn has_route(&self, route: &Route) -> bool {
            let origin = route.get_origin();
            if !self.has_as_paths_for_origin(origin) {
                return false;
            };
            self.get_as_paths_for_origin(origin).has_route(route)
        }

        pub fn insert_route(&mut self, route: Route) {
            debug!("Adding route {:#?}", route);
            if !self.has_route(&route) {
                self.add_origin(*route.get_origin());
                self.add_as_path(route.get_as_path().clone());
                self.add_route(route);
            }
        }

        pub fn merge_from(&mut self, other: &Self) {
            for origin in other.get_origins() {
                self.add_origin_as_paths(origin, other.get_as_paths_for_origin(origin));
            }
        }

        /// Merge pairs of PathData objs, delete the 2nd object from each pair,
        /// then merge the remaining objs in pairs. Continue until only one obj is left.
        pub fn merge_path_data(mut all_path_data: Vec<PathData>) -> PathData {
            info!("Merging {} objects", all_path_data.len());
            let origins: usize = all_path_data.iter().map(|d| d.count_origins()).sum();
            let as_paths: usize = all_path_data.iter().map(|d| d.count_as_paths()).sum();
            info!("Pre-merge, {} origins, {} AS paths", origins, as_paths);

            if all_path_data.is_empty() {
                panic!("No sequences to merge!");
            } else if all_path_data.len() == 1 {
                debug!("Only 1 item, nothing to merge");
                return all_path_data.pop().unwrap();
            }

            while all_path_data.len() > 1 {
                for chunks in all_path_data.chunks_mut(2) {
                    if let [seq1, seq2] = chunks {
                        seq1.merge_from(seq2);
                    }
                }

                let to_delete = all_path_data.len() / 2; // rounds down
                let mut deleted = 0;
                let mut index = 1;

                while deleted < to_delete {
                    for i in 0..all_path_data.len() {
                        // ^ Up to but not including

                        if i == index {
                            all_path_data.remove(i);
                            deleted += 1;
                            index += 1;
                            break;
                        }
                    }
                }
            }

            assert!(all_path_data.len() == 1);
            let path_data = all_path_data.pop().unwrap();
            info!(
                "Post-merge, {} origins, {} AS paths",
                path_data.count_origins(),
                path_data.count_as_paths()
            );

            path_data
        }

        fn remove_as_paths_for_origin(&mut self, origin: &Asn) {
            if self.has_as_paths_for_origin(origin) {
                debug!("Removing AS paths for origin {}", origin);
                self.as_paths.remove(origin);
            } else {
                panic!(
                    "Attempt to remove AS paths for non-existing origin {}",
                    origin
                );
            }
        }

        /// Remove origins which only have a single AS path
        pub fn remove_single_as_paths(&mut self) {
            info!("Removing single AS paths");

            let mut to_remove = Vec::new();
            for origin in self.get_origins() {
                if self.get_as_paths_for_origin(origin).len() == 1 {
                    to_remove.push(origin.to_owned());
                }
            }

            debug!("Removing {} origins with single AS paths", to_remove.len(),);
            for key in to_remove.iter() {
                self.remove_as_paths_for_origin(key);
            }

            info!(
                "Remaining multi-path origins {}, with {} AS paths",
                self.count_origins(),
                self.count_as_paths()
            );
        }
    }
}
