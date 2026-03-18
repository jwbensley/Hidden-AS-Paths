use crate::types::asn::Asn;
use crate::types::route::Route;
use bgpkit_parser::models::Asn as BgpKitAsn;
use log::debug;
use serde::ser::SerializeStruct as _;
use serde::{Serialize, Serializer};
use std::hash::Hash;

/// A deduped AS path which stores one or more routes and can be serialised to JSON.
#[derive(Debug, Clone, Eq, Default)]
pub struct AsPath {
    as_path: Vec<Asn>,
    routes: Vec<Route>,
}

impl Serialize for AsPath {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("AsPath", 2)?;
        state.serialize_field("as_path", &self.as_path)?;
        state.serialize_field("routes", &self.routes)?;
        state.end()
    }
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
    pub fn new(mut as_path: Vec<Asn>, routes: Vec<Route>) -> Self {
        as_path.dedup();
        AsPath { as_path, routes }
    }

    pub fn get_mock(as_path: Option<Vec<Asn>>) -> AsPath {
        let as_path = as_path.unwrap_or_else(|| {
            Vec::from([
                Asn::get_mock(Some(1)),
                Asn::get_mock(Some(2)),
                Asn::get_mock(Some(3)),
            ])
        });
        AsPath::new(as_path, Vec::new())
    }

    pub fn add_route(&mut self, route: Route) {
        if self.has_route(&route) {
            return;
        };
        self.routes.push(route);
    }

    pub fn get_asns(&self) -> &Vec<Asn> {
        &self.as_path
    }

    pub fn get_origin(&self) -> &Asn {
        self.as_path.last().unwrap()
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

    pub fn from_vec(asns: &[BgpKitAsn]) -> Self {
        let as_path = asns
            .iter()
            .map(|a| Asn::new(a.to_u32()))
            .collect::<Vec<Asn>>();
        AsPath::new(as_path, Vec::new())
    }

    /// The same ASN appears somewhere in both AS Paths and before the origin ASN.
    /// From the point of this shared ASN to the origin, the path must be different.
    /// E.g., extra ASNs are in the path:
    /// a = [1, 2, 3]
    /// b = [4, 2, 5, 3]
    ///            ^
    /// Or, a different path is taken:
    /// a = [1, 2, 5, 3]
    /// b = [4, 2, 6, 3]
    ///            ^
    /// Or, ASNs are missing from the path:
    /// a = [1, 2, 5, 3]
    /// b = [4, 2, 3]
    ///            ^
    pub fn is_divergent_with(&self, other: &AsPath) -> bool {
        assert_eq!(
            self.get_origin(),
            other.get_origin(),
            "The origin must be the same to compare AS paths for divergence"
        );

        // Compare the paths up to he origin, the origin must be the same
        let a_path = self.get_asns().split_last().unwrap().1;
        let b_path = other.get_asns().split_last().unwrap().1;

        for a_asn in a_path {
            let a_pos = a_path.iter().position(|x| x == a_asn).unwrap();
            let b_pos = b_path.iter().position(|x| x == a_asn);

            if let Some(b_pos) = b_pos
                // The remainder of each path after the shared ASN must be different
                && a_path[a_pos..] != b_path[b_pos..]
            // && (a_path.len() - a_pos != b_path.len() - b_pos)
            {
                return true;
            }
        }
        false
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
        ap_1 = AsPath::get_mock(Some(Vec::from([Asn::new(1)])));
        ap_1.add_route(Route::get_mock(Some(ap_1.clone())));
        ap_2 = AsPath::get_mock(Some(Vec::from([Asn::new(1)])));
        ap_2.add_route(Route::get_mock(Some(ap_2.clone())));
        assert_eq!(ap_1, ap_2);
    }

    #[test]
    fn test_as_path_ne() {
        // NE with different origins
        let mut ap_1 = AsPath::get_mock(Some(Vec::from([Asn::new(1)])));
        let mut ap_2 = AsPath::get_mock(Some(Vec::from([Asn::new(2)])));
        assert_ne!(ap_1, ap_2);

        // NE if missing Route
        ap_1 = AsPath::get_mock(Some(Vec::from([Asn::new(1)])));
        ap_1.add_route(Route::get_mock(Some(ap_1.clone())));
        ap_2 = AsPath::get_mock(Some(Vec::from([Asn::new(1)])));
        assert_ne!(ap_1, ap_2);

        // NE if different routes
        ap_1 = AsPath::get_mock(Some(Vec::from([Asn::new(1)])));
        ap_1.add_route(Route::get_mock(Some(ap_1.clone())));
        ap_2 = AsPath::get_mock(Some(Vec::from([Asn::new(1)])));
        ap_1.add_route(Route::get_mock(Some(AsPath::get_mock(Some(Vec::from([
            Asn::new(2),
        ]))))));
        assert_ne!(ap_1, ap_2);
    }

    // #[test]
    // fn test_has_divergence_with() {
    //     // Shared ASNs - no divergent paths
    //     let ap_1 = AsPath::get_mock(None);
    //     let ap_2 = AsPath::get_mock(None);
    //     assert!(ap_1.len() >= 3);
    //     assert_eq!(ap_1.get_asns(), ap_2.get_asns());
    //     assert!(!ap_1.has_divergence_with(&ap_2));

    //     // Shared ASNs - divergent paths
    //     let mut path_2: Vec<Asn> = ap_1.get_asns().clone();
    //     path_2.insert(ap_1.len() - 1, Asn::new(23456));
    //     let ap_2 = AsPath::new(path_2);
    //     assert_ne!(ap_1.get_asns(), ap_2.get_asns());
    //     assert!(ap_1.len() >= 3);
    //     assert!(ap_2.len() >= 3);
    //     assert!(ap_1.has_divergence_with(&ap_2));

    //     // No shared ASNs - no divergent paths
    //     let ap_1 = AsPath::new(Vec::from([
    //         Asn::new(1),
    //         Asn::new(2),
    //         Asn::new(3),
    //     ]));
    //     let ap_2 = AsPath::new(Vec::from([
    //         Asn::new(4),
    //         Asn::new(5),
    //         Asn::new(6),
    //     ]));
    //     assert_ne!(ap_1.get_asns(), ap_2.get_asns());
    //     assert!(ap_1.len() == 3);
    //     assert!(ap_2.len() == 3);
    //     assert!(!ap_1.has_divergence_with(&ap_2));
    // }
}
