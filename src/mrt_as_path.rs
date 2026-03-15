/// A deduped AS path which stores one or more routes and can be serialised to JSON.
pub mod mrt_as_path {
    use crate::{mrt_asn::asn::MrtAsn, mrt_route::route::Route};
    use log::debug;
    use serde::ser::SerializeStruct as _;
    use serde::{Serialize, Serializer};
    use std::hash::Hash;

    #[derive(Debug, Clone, Eq)]
    pub struct MrtAsPath {
        as_path: Vec<MrtAsn>,
        routes: Vec<Route>,
    }

    impl Serialize for MrtAsPath {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let mut state = serializer.serialize_struct("MrtAsPath", 2)?;
            state.serialize_field("as_path", &self.as_path)?;
            state.serialize_field("routes", &self.routes)?;
            state.end()
        }
    }

    impl PartialEq for MrtAsPath {
        fn eq(&self, other: &Self) -> bool {
            (self.routes == other.routes) && (self.as_path == other.as_path)
        }
    }

    impl Hash for MrtAsPath {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            self.as_path.hash(state);
            self.routes.hash(state);
        }
    }

    impl MrtAsPath {
        pub fn new(mut as_path: Vec<MrtAsn>, routes: Vec<Route>) -> Self {
            as_path.dedup();
            MrtAsPath {
                as_path,
                routes,
            }
        }

        pub fn default() -> Self {
            MrtAsPath::new(Vec::new(), Vec::new())
        }

        pub fn get_mock(as_path: Option<Vec<MrtAsn>>) -> MrtAsPath {
            let as_path = as_path.unwrap_or_else(|| Vec::from([
                MrtAsn::get_mock(Some(1)),
                MrtAsn::get_mock(Some(2)),
                MrtAsn::get_mock(Some(3))]));
            MrtAsPath::new(as_path, Vec::new())
        }

        pub fn add_route(&mut self, route: Route) {
            if self.has_route(&route) {
                return;
            };
            self.routes.push(route);
        }

        pub fn get_as_path(&self) -> &Vec<MrtAsn> {
            &self.as_path
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

        /// The same ASN appears somewhere in both AS Paths (not the final, origin ASN),
        /// From the point of this shared ASN to the origin, the path must be different:
        /// a = [1, 2, 3]
        /// b = [4, 2, 5, 3]
        ///         ^  ^
        // pub fn has_divergence_with(&self, other: &MrtAsPath) -> bool {
        //     let a_path = self.get_asns().split_last().unwrap().1;
        //     let b_path = other.get_asns().split_last().unwrap().1;

        //     for a_asn in a_path {
        //         let a_pos = a_path.iter().position(|x| x == a_asn).unwrap();
        //         let b_pos = b_path.iter().position(|x| x == a_asn);

        //         if let Some(b_pos) = b_pos
        //             && a_path[a_pos..] != b_path[b_pos..]
        //             && (a_path.len() - a_pos != b_path.len() - b_pos)
        //         {
        //             return true;
        //         }
        //     }
        //     false
        // }
        
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_as_path_eq() {
            // EQ with same default origin and default AS path
            let mut ap_1 = MrtAsPath::get_mock(None);
            ap_1.add_route(Route::get_mock(None));
            let mut ap_2 = MrtAsPath::get_mock(None);
            ap_2.add_route(Route::get_mock(None));
            assert_eq!(ap_1, ap_2);

            // EQ with same explicit origin and explicit AS path
            ap_1 = MrtAsPath::get_mock(Some(Vec::from([MrtAsn::new(1)])));
            ap_1.add_route(Route::get_mock(Some(ap_1.clone())));
            ap_2 = MrtAsPath::get_mock(Some(Vec::from([MrtAsn::new(1)])));
            ap_2.add_route(Route::get_mock(Some(ap_2.clone())));
            assert_eq!(ap_1, ap_2);
        }

        #[test]
        fn test_as_path_ne() {
            // NE with different origins
            let mut ap_1 = MrtAsPath::get_mock(Some(Vec::from([MrtAsn::new(1)])));
            let mut ap_2 = MrtAsPath::get_mock(Some(Vec::from([MrtAsn::new(2)])));
            assert_ne!(ap_1, ap_2);

            // NE if missing Route
            ap_1 = MrtAsPath::get_mock(Some(Vec::from([MrtAsn::new(1)])));
            ap_1.add_route(Route::get_mock(Some(ap_1.clone())));
            ap_2 = MrtAsPath::get_mock(Some(Vec::from([MrtAsn::new(1)])));
            assert_ne!(ap_1, ap_2);

            // NE if different routes
            ap_1 = MrtAsPath::get_mock(Some(Vec::from([MrtAsn::new(1)])));
            ap_1.add_route(Route::get_mock(Some(ap_1.clone())));
            ap_2 = MrtAsPath::get_mock(Some(Vec::from([MrtAsn::new(1)])));
            ap_1.add_route(Route::get_mock(Some(MrtAsPath::get_mock(Some(Vec::from([MrtAsn::new(2)]))))));
            assert_ne!(ap_1, ap_2);
        }

        // #[test]
        // fn test_has_divergence_with() {
        //     // Shared ASNs - no divergent paths
        //     let ap_1 = MrtAsPath::get_mock(None);
        //     let ap_2 = MrtAsPath::get_mock(None);
        //     assert!(ap_1.len() >= 3);
        //     assert_eq!(ap_1.get_as_path(), ap_2.get_as_path());
        //     assert!(!ap_1.has_divergence_with(&ap_2));

        //     // Shared ASNs - divergent paths
        //     let mut path_2: Vec<MrtAsn> = ap_1.get_as_path().clone();
        //     path_2.insert(ap_1.len() - 1, MrtAsn::new(23456));
        //     let ap_2 = MrtAsPath::new(path_2);
        //     assert_ne!(ap_1.get_as_path(), ap_2.get_as_path());
        //     assert!(ap_1.len() >= 3);
        //     assert!(ap_2.len() >= 3);
        //     assert!(ap_1.has_divergence_with(&ap_2));

        //     // No shared ASNs - no divergent paths
        //     let ap_1 = MrtAsPath::new(Vec::from([
        //         MrtAsn::new(1),
        //         MrtAsn::new(2),
        //         MrtAsn::new(3),
        //     ]));
        //     let ap_2 = MrtAsPath::new(Vec::from([
        //         MrtAsn::new(4),
        //         MrtAsn::new(5),
        //         MrtAsn::new(6),
        //     ]));
        //     assert_ne!(ap_1.get_as_path(), ap_2.get_as_path());
        //     assert!(ap_1.len() == 3);
        //     assert!(ap_2.len() == 3);
        //     assert!(!ap_1.has_divergence_with(&ap_2));
        // }
    }
}
