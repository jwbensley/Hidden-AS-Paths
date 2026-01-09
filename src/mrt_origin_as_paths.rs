pub mod origin_as_paths {
    use crate::mrt_as_path::as_path::AsPath;
    use crate::mrt_asn::asn::Testing;
    use crate::mrt_route::route::Route;
    use bgpkit_parser::models::Asn;
    use log::debug;
    use std::collections::HashMap;

    /// A vector of unique, deduped AS paths, which all point to the same origin ASN
    #[derive(Debug, Clone)]
    pub struct OriginAsPaths {
        origin: Asn,
        as_paths: Vec<AsPath>,
    }

    impl PartialEq for OriginAsPaths {
        fn eq(&self, other: &Self) -> bool {
            (self.as_paths == other.as_paths) && (self.origin == other.origin)
        }
    }

    impl OriginAsPaths {
        pub fn new(origin: Asn) -> Self {
            OriginAsPaths {
                origin,
                as_paths: Vec::<AsPath>::new(),
            }
        }

        pub fn get_mock(origin: Option<Asn>) -> OriginAsPaths {
            let o = origin.unwrap_or(Asn::get_mock(None));
            OriginAsPaths {
                origin: o,
                as_paths: Vec::from([AsPath::get_mock(Some(o))]),
            }
        }

        pub fn add_as_path(&mut self, as_path: AsPath) {
            if self.has_as_path(&as_path) {
                return;
            };
            self.as_paths.push(as_path);
        }

        pub fn add_route(&mut self, route: Route) {
            let as_path = AsPath::new(route.get_as_path().clone());
            self.get_as_path_mut(&as_path).add_route(route);
        }

        pub fn find_overlapping_paths(&self) {
            let overlapping_paths: HashMap<AsPath, Vec<AsPath>> =
                HashMap::<AsPath, Vec<AsPath>>::new();

            for a in self.get_as_paths() {
                for b in self.get_as_paths() {
                    if a == b {
                        continue;
                    };
                    if a.has_overlap_with(b) {
                        //overlapping_paths.push()
                    }
                }
            }
        }

        fn get_as_paths(&self) -> &Vec<AsPath> {
            &self.as_paths
        }

        fn get_as_paths_mut(&mut self) -> &mut Vec<AsPath> {
            self.as_paths.as_mut()
        }

        fn get_as_path(&self, as_path: &AsPath) -> &AsPath {
            for a in self.get_as_paths() {
                if a.get_as_path() == as_path.get_as_path() {
                    return a;
                }
            }
            panic!("AS Path not found {:#?}", as_path);
        }

        fn get_as_path_mut(&mut self, as_path: &AsPath) -> &mut AsPath {
            for a in self.get_as_paths_mut() {
                if a.get_as_path() == as_path.get_as_path() {
                    return a;
                }
            }
            panic!("AS Path not found {:#?}", as_path);
        }

        pub fn get_origin(&self) -> &Asn {
            &self.origin
        }

        pub fn has_as_path(&self, as_path: &AsPath) -> bool {
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
            let as_path = AsPath::new(route.get_as_path().clone());
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

        fn remove_as_path(&mut self, as_path: &AsPath) {
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

            let oap_1 = OriginAsPaths::get_mock(Some(Asn::new_32bit(1)));
            let oap_2 = OriginAsPaths::get_mock(Some(Asn::new_32bit(1)));
            assert_eq!(oap_1, oap_2);
        }

        #[test]
        fn test_origin_as_paths_ne() {
            let oap_1 = OriginAsPaths::get_mock(Some(Asn::new_32bit(1)));
            let oap_2 = OriginAsPaths::get_mock(Some(Asn::new_32bit(2)));
            assert_ne!(oap_1, oap_2);
        }
    }
}
