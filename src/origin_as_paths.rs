/// This module implements a data structure for storing AS paths, grouped by their origin ASN.
pub mod origin_aspaths {
    use crate::mrt_as_path::mrt_as_path::MrtAsPath;
    use crate::mrt_asn::asn::MrtAsn;
    use crate::mrt_route::route::Route;
    use log::debug;
    use std::collections::HashMap;

    /// A vector of AS paths which all point to the same origin ASN
    #[derive(Debug, Clone)]
    pub struct OriginAsPaths {
        origin: MrtAsn,
        as_paths: Vec<MrtAsPath>,
    }

    impl PartialEq for OriginAsPaths {
        fn eq(&self, other: &Self) -> bool {
            (self.as_paths == other.as_paths) && (self.origin == other.origin)
        }
    }

    impl OriginAsPaths {
        pub fn new(origin: MrtAsn, as_paths: Vec<MrtAsPath>) -> Self {
            OriginAsPaths { origin, as_paths }
        }

        pub fn default(origin: MrtAsn) -> Self {
            OriginAsPaths::new(origin, Vec::<MrtAsPath>::new())
        }

        pub fn get_mock(origin: Option<MrtAsn>) -> OriginAsPaths {
            let o = origin.unwrap_or(MrtAsn::get_mock(None));
            OriginAsPaths::new(o, Vec::from([MrtAsPath::get_mock(Some(o))]))
        }

        pub fn add_as_path(&mut self, as_path: MrtAsPath) {
            if !self.has_as_path(&as_path) {
                self.as_paths.push(as_path);
            };
        }

        pub fn add_route(&mut self, route: Route) {
            let as_path = route.get_as_path().clone();
            self.get_as_path_mut(&as_path).add_route(route);
        }

        // pub fn find_divergent_paths(&self) -> HashMap<MrtAsPath, Vec<&MrtAsPath>> {
        //     let mut divergent_paths: HashMap<MrtAsPath, Vec<&MrtAsPath>> = HashMap::new();

        //     let mut checked = Vec::<&MrtAsPath>::new();

        //     for a in self.get_as_paths() {
        //         for b in self.get_as_paths() {
        //             if a == b {
        //                 continue;
        //             };
        //             if checked.contains(&a) {
        //                 continue;
        //             }
        //             if a.has_divergence_with(b) {
        //                 if !divergent_paths.contains_key(a) {
        //                     divergent_paths.insert(a.clone(), Vec::new());
        //                 };
        //                 divergent_paths.get_mut(a).unwrap().push(b);
        //             }
        //         }
        //         checked.push(a);
        //     }
        //     divergent_paths
        // }

        fn get_as_paths(&self) -> &Vec<MrtAsPath> {
            &self.as_paths
        }

        fn get_as_paths_mut(&mut self) -> &mut Vec<MrtAsPath> {
            self.as_paths.as_mut()
        }

        fn get_as_path(&self, as_path: &MrtAsPath) -> &MrtAsPath {
            for a in self.get_as_paths() {
                if a.get_as_path() == as_path.get_as_path() {
                    return a;
                }
            }
            panic!("AS Path not found {:#?}", as_path);
        }

        fn get_as_path_mut(&mut self, as_path: &MrtAsPath) -> &mut MrtAsPath {
            for a in self.get_as_paths_mut() {
                if a.get_as_path() == as_path.get_as_path() {
                    return a;
                }
            }
            panic!("AS Path not found {:#?}", as_path);
        }

        pub fn get_origin(&self) -> &MrtAsn {
            &self.origin
        }

        pub fn has_as_path(&self, as_path: &MrtAsPath) -> bool {
            for a in self.get_as_paths() {
                if a == as_path {
                    debug!("AS path found: {:#?}", as_path);
                    return true;
                }
            }
            debug!("AS path not found: {:#?}", as_path);
            false
        }

        pub fn has_route(&self, route: &Route) -> bool {
            if route.get_origin() != self.get_origin() {
                panic!(
                    "Checking if route exists in AS Paths for origin {}: {:#?}",
                    self.get_origin(),
                    route
                )
            };
            let as_path = route.get_as_path().clone();
            if !self.has_as_path(&as_path) {
                return false;
            };
            let as_path = self.get_as_path(&as_path);
            as_path.has_route(route)
        }

        pub fn is_empty(&self) -> bool {
            self.len() == 0
        }

        pub fn len(&self) -> usize {
            self.as_paths.len()
        }

        pub fn merge_from(&mut self, other: &Self) {
            for as_path in other.get_as_paths() {
                self.add_as_path(as_path.clone());
            }
        }

        fn remove_as_path(&mut self, as_path: &MrtAsPath) {
            self.as_paths
                .remove(self.as_paths.iter().position(|x| x == as_path).unwrap());
        }

        pub fn remove_single_hop_paths(&mut self) {
            let mut to_remove = Vec::new();
            for as_path in self.get_as_paths() {
                if as_path.len() == 1 {
                    to_remove.push(as_path.clone());
                }
            }
            debug!("Single-hop AS Paths to remove: {}", to_remove.len());
            for as_path in to_remove {
                self.remove_as_path(&as_path);
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_origin_as_paths_eq() {
            let oap_1 = OriginAsPaths::get_mock(None);
            let oap_2 = OriginAsPaths::get_mock(None);
            assert_eq!(oap_1, oap_2);

            let oap_1 = OriginAsPaths::get_mock(Some(MrtAsn::new(1)));
            let oap_2 = OriginAsPaths::get_mock(Some(MrtAsn::new(1)));
            assert_eq!(oap_1, oap_2);
        }

        #[test]
        fn test_origin_as_paths_ne() {
            let oap_1 = OriginAsPaths::get_mock(Some(MrtAsn::new(1)));
            let oap_2 = OriginAsPaths::get_mock(Some(MrtAsn::new(2)));
            assert_ne!(oap_1, oap_2);
        }
    }
}
