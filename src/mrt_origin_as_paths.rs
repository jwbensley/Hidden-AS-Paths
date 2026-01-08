pub mod origin_as_paths {
    use crate::mrt_as_path::as_path::AsPath;
    use crate::mrt_route::route::Route;
    use bgpkit_parser::models::Asn;
    use log::debug;

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
            let mut present = false;
            for a in self.get_as_paths() {
                if a.get_as_path() == as_path.get_as_path() {
                    present = true;
                    break;
                }
            }
            debug!("AS path present {}: {:#?}", present, as_path);
            present
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
    }
}
