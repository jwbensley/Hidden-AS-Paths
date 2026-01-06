pub mod path_data {
    use bgpkit_parser::models::Asn;
    use log::{debug, info};
    use std::collections::HashMap;
    use std::collections::hash_map::Keys;
    use crate::as_paths::as_paths::{OriginAsPaths, Route};

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
            let origin_as_paths = self.get_as_paths_for_origin_mut(&as_path.last().unwrap());
            if origin_as_paths.has_as_path(&as_path) { return };
            origin_as_paths.add_as_path(as_path)
        }

        fn add_origin(&mut self, origin: Asn) {
            if self.has_as_paths_for_origin(&origin) { return };
            self.as_paths.insert(origin.clone(), OriginAsPaths::new(origin));
        }

        fn add_route(&mut self, route: Route) {
            self.get_as_paths_for_origin_mut(&route.get_origin()).add_route(route);
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
            if !self.has_as_paths_for_origin(origin) { return false; };
            self.get_as_paths_for_origin(origin).has_route(route)
        }

        pub fn insert_route(
            &mut self,
            route: Route,
        ) {
            debug!(
                "Adding route {:?}",
                route
            );
            if !self.has_route(&route) {
                self.add_origin(route.get_origin().clone());
                self.add_as_path(route.get_as_path().clone());
                self.add_route(route);
            }
        }

    //     pub fn get_as_paths_at_sequence(&self, sequence: &Vec<Asn>) -> &AsPaths {
    //         self.sequences.get(sequence).unwrap()
    //     }

    //     pub fn get_sequences(&self) -> Keys<'_, Vec<Asn>, AsPaths> {
    //         self.sequences.keys()
    //     }

    //     pub fn print_as_paths(&self) {
    //         for sequence in self.get_sequences() {
    //             info!("{:?}", sequence);
    //             for path in self.get_as_paths_at_sequence(sequence).get_paths() {
    //                 info!("    {:?}", path);
    //             }
    //         }
    //     }

    //     pub fn print_total(&self) {
    //         let mut sequences = 0;
    //         let mut paths = 0;
    //         let mut routes = 0;

    //         for sequence in self.get_sequences() {
    //             sequences += 1;
    //             let as_paths = self.get_as_paths_at_sequence(sequence);
    //             for path in as_paths.get_paths() {
    //                 paths += 1;
    //                 routes += as_paths.get_routes_at_path(path).len();
    //             }
    //         }
    //         info!(
    //             "{} deduped paths, {} paths, {} routes",
    //             sequences, paths, routes
    //         );
    //     }

    //     pub fn remove_single_paths(&mut self) {
    //         /*
    //          * Remove all deduped AS sequences which only have a single AS path
    //          */
    //         let mut to_remove = Vec::new();
    //         for sequence in self.get_sequences() {
    //             if self.get_as_paths_at_sequence(sequence).len() == 1 {
    //                 to_remove.push(sequence.to_owned());
    //             }
    //         }
    //         for key in to_remove.iter() {
    //             self.sequences.remove(key);
    //         }
    //     }
    // }

        pub fn merge_from(&mut self, other: &Self) {
            for origin in other.get_origins(){
                let origin_as_paths = other.get_as_paths_for_origin(origin);
                
            }
        
            for other_sequence in other.get_as_paths_for_origin(origin) {
                let other_as_paths = other.get_as_paths_at_sequence(other_sequence);

                for other_as_path in other_as_paths.get_paths() {
                    let other_routes = other_as_paths.get_routes_at_path(&other_as_path);

                    for route in other_routes.get_routes() {
                        self.insert_route_at_sequence(
                            other_sequence.clone(),
                            other_as_path.clone(),
                            route.clone(),
                        );
                    }
                }
            }
        }
    
        /// Merge pairs of PathData objs, delete the 2nd object from each pair,
        /// then merge the remaining objs in pairs. Continue until only one obj is left.
        pub fn merge_path_data(mut all_path_data: Vec<PathData>) -> PathData {
            info!("Merging {} objects", all_path_data.len());

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
            all_path_data.pop().unwrap()

        }

    }

}
