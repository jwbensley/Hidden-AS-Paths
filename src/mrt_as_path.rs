pub mod as_path {
    use crate::mrt_asn::asn::Testing;
    use crate::mrt_route::route::Route;
    use bgpkit_parser::models::Asn;
    use log::debug;
    use std::hash::Hash;

    /// A deduped AS path which stores one or more routes
    #[derive(Debug, Clone, Eq)]
    pub struct AsPath {
        as_path: Vec<Asn>,
        routes: Vec<Route>,
    }

    impl PartialEq for AsPath {
        fn eq(&self, other: &Self) -> bool {
            (self.routes == other.routes) && (self.as_path == other.as_path)
        }
    }

    impl Hash for AsPath {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            self.as_path.hash(state);
            self.routes.hash(state);
        }
    }

    impl AsPath {
        pub fn new(mut as_path: Vec<Asn>) -> Self {
            as_path.dedup();
            AsPath {
                as_path,
                routes: Vec::<Route>::new(),
            }
        }

        pub fn get_mock(origin: Option<Asn>) -> AsPath {
            let as_path = Vec::from([
                Asn::get_mock(Some(1)),
                Asn::get_mock(Some(2)),
                origin.unwrap_or(Asn::get_mock(None)),
            ]);
            AsPath::new(as_path)
        }

        pub fn add_route(&mut self, route: Route) {
            if self.has_route(&route) {
                return;
            };
            self.routes.push(route);
        }

        pub fn get_as_path(&self) -> &Vec<Asn> {
            &self.as_path
        }

        fn get_asns(&self) -> &Vec<Asn> {
            &self.as_path
        }

        /// The same ASN appears somewhere in both AS Paths (not the final, origin, ASN),
        /// that is gaurenteed to be the same ASN. From the point of this shared ASN to
        /// the origin, the path must be different:
        /// a = [1, 2, 3]
        /// b = [4, 2, 5, 3]
        ///         ^  ^
        pub fn has_divergence_with(&self, other: &AsPath) -> bool {
            let a_path = self.get_asns().split_last().unwrap().1;
            let b_path = other.get_asns().split_last().unwrap().1;

            for a_asn in a_path {
                let a_pos = a_path.iter().position(|x| x == a_asn).unwrap();
                let b_pos = b_path.iter().position(|x| x == a_asn);

                if b_pos.is_some() && a_path[a_pos..] != b_path[b_pos.unwrap()..] {
                    return true;
                }
            }
            false
        }

        pub fn has_route(&self, route: &Route) -> bool {
            let present = self.routes.contains(route);
            debug!("Route present {:#?}: {}", route, present);
            present
        }

        pub fn is_empty(&self) -> bool {
            self.len() == 0
        }

        pub fn len(&self) -> usize {
            self.as_path.len()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_as_path_eq() {
            // EQ with same default origin and default AS path
            let mut ap_1 = AsPath::get_mock(None);
            ap_1.add_route(Route::get_mock(None));
            let mut ap_2 = AsPath::get_mock(None);
            ap_2.add_route(Route::get_mock(None));
            assert_eq!(ap_1, ap_2);

            // EQ with same explicit origin and explicit AS path
            ap_1 = AsPath::get_mock(Some(Asn::new_32bit(1)));
            ap_1.add_route(Route::get_mock(Some(Asn::new_32bit(1))));
            ap_2 = AsPath::get_mock(Some(Asn::new_32bit(1)));
            ap_2.add_route(Route::get_mock(Some(Asn::new_32bit(1))));
            assert_eq!(ap_1, ap_2);
        }

        #[test]
        fn test_as_path_ne() {
            // NE with different origins
            let mut ap_1 = AsPath::get_mock(Some(Asn::new_32bit(1)));
            let mut ap_2 = AsPath::get_mock(Some(Asn::new_32bit(2)));
            assert_ne!(ap_1, ap_2);

            // NE if missing Route
            ap_1 = AsPath::get_mock(Some(Asn::new_32bit(1)));
            ap_1.add_route(Route::get_mock(Some(Asn::new_32bit(1))));
            ap_2 = AsPath::get_mock(Some(Asn::new_32bit(1)));
            assert_ne!(ap_1, ap_2);

            // NE if different routes
            ap_1 = AsPath::get_mock(Some(Asn::new_32bit(1)));
            ap_1.add_route(Route::get_mock(Some(Asn::new_32bit(1))));
            ap_2 = AsPath::get_mock(Some(Asn::new_32bit(1)));
            ap_1.add_route(Route::get_mock(Some(Asn::new_32bit(2))));
            assert_ne!(ap_1, ap_2);
        }

        #[test]
        fn test_has_divergence_with() {
            // Shared ASNs - no divergent paths
            let ap_1 = AsPath::get_mock(None);
            let ap_2 = AsPath::get_mock(None);
            assert!(ap_1.len() >= 3);
            assert_eq!(ap_1.get_as_path(), ap_2.get_as_path());
            assert!(!ap_1.has_divergence_with(&ap_2));

            // Shared ASNs - divergent paths
            let mut path_2: Vec<Asn> = ap_1.get_as_path().clone();
            path_2.insert(ap_1.len() - 1, Asn::new_32bit(23456));
            let ap_2 = AsPath::new(path_2);
            assert_ne!(ap_1.get_as_path(), ap_2.get_as_path());
            assert!(ap_1.len() >= 3);
            assert!(ap_2.len() >= 3);
            assert!(ap_1.has_divergence_with(&ap_2));

            // No shared ASNs - no divergent paths
            let ap_1 = AsPath::new(Vec::from([
                Asn::new_32bit(1),
                Asn::new_32bit(2),
                Asn::new_32bit(3),
            ]));
            let ap_2 = AsPath::new(Vec::from([
                Asn::new_32bit(4),
                Asn::new_32bit(5),
                Asn::new_32bit(6),
            ]));
            assert_ne!(ap_1.get_as_path(), ap_2.get_as_path());
            assert!(ap_1.len() == 3);
            assert!(ap_2.len() == 3);
            assert!(!ap_1.has_divergence_with(&ap_2));
        }
    }
}
