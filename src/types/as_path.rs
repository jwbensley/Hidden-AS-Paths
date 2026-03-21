use crate::types::asn::Asn;
use crate::types::route::Route;
use bgpkit_parser::models::Asn as BgpKitAsn;
use core::panic;
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

    pub fn get_mock(as_path: Option<Vec<Asn>>, routes: Option<Vec<Route>>) -> AsPath {
        let as_path = as_path.unwrap_or_else(|| {
            Vec::from([
                Asn::get_mock(Some(3)),
                Asn::get_mock(Some(2)),
                Asn::get_mock(None),
            ])
        });
        let routes = routes.unwrap_or_else(|| Vec::from([Route::get_mock(None)]));
        AsPath::new(as_path, routes)
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
        if self.as_path.is_empty() {
            panic!("AS path is empty, cannot get origin ASN");
        }
        self.as_path.last().unwrap()
    }

    pub fn get_routes(&self) -> &Vec<Route> {
        &self.routes
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
    /// Or, a = [1, 2, 5, 3]
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
    fn test_new() {
        let as_path = vec![Asn::new(1), Asn::new(2), Asn::new(3)];
        let routes = vec![Route::get_mock(None)];
        let ap = AsPath::new(as_path.clone(), routes.clone());

        assert_eq!(ap.len(), as_path.len());
        assert_eq!(ap.get_asns(), &as_path);
        assert_eq!(ap.routes.len(), 1);
        assert_eq!(ap.routes[0], routes[0]);
    }

    #[test]
    fn test_new_dedups_consecutive_asns() {
        let as_path = vec![
            Asn::new(1),
            Asn::new(1),
            Asn::new(2),
            Asn::new(2),
            Asn::new(3),
        ];
        let ap = AsPath::new(as_path, Vec::new());

        assert_eq!(ap.len(), 3);
        assert_eq!(ap.get_asns(), &vec![Asn::new(1), Asn::new(2), Asn::new(3)]);
    }

    #[test]
    fn test_get_mock() {
        let ap = AsPath::get_mock(None, None);
        assert_eq!(ap.len(), 3);
        assert_eq!(ap.get_asns(), &vec![Asn::new(3), Asn::new(2), Asn::new(1)]);
        assert_eq!(ap.routes.len(), 0);
    }

    #[test]
    fn test_get_mock_custom_path() {
        let custom_path = vec![Asn::new(10), Asn::new(20), Asn::new(30)];
        let ap = AsPath::get_mock(Some(custom_path.clone()), None);
        assert_eq!(ap.get_asns(), &custom_path);
    }

    #[test]
    fn test_add_route() {
        let mut ap = AsPath::get_mock(None, None);
        let route_1 = Route::get_mock(None);
        assert_eq!(ap.routes.len(), 1);
        assert!(ap.has_route(&route_1));

        let route_2 = Route::get_mock(Some(AsPath::get_mock(Some(vec![Asn::new(999)]), None)));
        ap.add_route(route_2.clone());
        assert_eq!(ap.routes.len(), 2);
    }

    #[test]
    fn test_add_route_duplicate() {
        let mut ap = AsPath::get_mock(None, None);
        let route = Route::get_mock(None);

        ap.add_route(route.clone());
        ap.add_route(route.clone());

        assert_eq!(ap.routes.len(), 1);
        assert!(ap.has_route(&route));
    }

    #[test]
    fn test_add_multiple_different_routes() {
        let mut ap = AsPath::get_mock(None, None);
        let route1 = Route::get_mock(Some(AsPath::get_mock(Some(vec![Asn::new(1)]), None)));
        let route2 = Route::get_mock(Some(AsPath::get_mock(Some(vec![Asn::new(2)]), None)));

        ap.add_route(route1.clone());
        ap.add_route(route2.clone());

        assert_eq!(ap.routes.len(), 2);
        assert!(ap.has_route(&route1));
        assert!(ap.has_route(&route2));
    }

    #[test]
    fn test_get_asns_empty() {
        let ap = AsPath::new(Vec::new(), Vec::new());
        assert_eq!(ap.get_asns(), &Vec::new());
    }

    #[test]
    fn test_get_asns() {
        let as_path = vec![Asn::new(100), Asn::new(200), Asn::new(300)];
        let ap = AsPath::new(as_path.clone(), Vec::new());
        assert_eq!(ap.get_asns(), &as_path);
    }

    #[test]
    fn test_get_origin_empty() {
        assert!(
            std::panic::catch_unwind(|| {
                let ap = AsPath::new(Vec::new(), Vec::new());
                ap.get_origin();
            })
            .is_err()
        );
    }

    #[test]
    fn test_get_origin() {
        let as_path = vec![Asn::new(1), Asn::new(2), Asn::new(3)];
        let ap = AsPath::new(as_path, Vec::new());
        assert_eq!(ap.get_origin(), &Asn::new(3));
    }

    #[test]
    fn test_get_origin_single_asn() {
        let as_path = vec![Asn::new(42)];
        let ap = AsPath::new(as_path, Vec::new());
        assert_eq!(ap.get_origin(), &Asn::new(42));
    }

    #[test]
    fn test_get_routes_empty() {
        let ap = AsPath::new(Vec::new(), Vec::new());
        assert_eq!(ap.get_routes(), &Vec::new());
    }

    #[test]
    fn test_get_routes() {
        let route1 = Route::get_mock(None);
        let route2 = Route::get_mock(None);
        let ap = AsPath::new(Vec::new(), vec![route1.clone(), route2.clone()]);
        assert_eq!(ap.get_routes(), &vec![route1, route2]);
    }

    #[test]
    fn test_has_route_true() {
        let mut ap = AsPath::get_mock(None, None);
        let route = Route::get_mock(None);
        ap.add_route(route.clone());
        assert!(ap.has_route(&route));
    }

    #[test]
    fn test_has_route_false() {
        let ap = AsPath::get_mock(None, None);
        let route = Route::get_mock(None);
        assert!(!ap.has_route(&route));
    }

    #[test]
    fn test_is_empty_true() {
        let ap = AsPath::new(Vec::new(), Vec::new());
        assert!(ap.is_empty());
    }

    #[test]
    fn test_is_empty_false() {
        let ap = AsPath::get_mock(None, None);
        assert!(!ap.is_empty());
    }

    #[test]
    fn test_len() {
        let ap = AsPath::new(vec![Asn::new(1), Asn::new(2), Asn::new(3)], Vec::new());
        assert_eq!(ap.len(), 3);

        let ap_empty = AsPath::new(Vec::new(), Vec::new());
        assert_eq!(ap_empty.len(), 0);
    }

    #[test]
    fn test_from_vec() {
        let bgpkit_asns = vec![
            BgpKitAsn::new_32bit(100),
            BgpKitAsn::new_32bit(200),
            BgpKitAsn::new_32bit(300),
        ];

        let ap = AsPath::from_vec(&bgpkit_asns);

        assert_eq!(ap.len(), 3);
        assert_eq!(
            ap.get_asns(),
            &vec![Asn::new(100), Asn::new(200), Asn::new(300)]
        );
        assert_eq!(ap.routes.len(), 0);
    }

    #[test]
    fn test_from_vec_empty() {
        let bgpkit_asns: Vec<BgpKitAsn> = Vec::new();
        let ap = AsPath::from_vec(&bgpkit_asns);

        assert_eq!(ap.len(), 0);
        assert!(ap.is_empty());
    }

    #[test]
    fn test_is_divergent_with_same_path() {
        let ap1 = AsPath::new(vec![Asn::new(1), Asn::new(2), Asn::new(3)], Vec::new());
        let ap2 = AsPath::new(vec![Asn::new(1), Asn::new(2), Asn::new(3)], Vec::new());

        assert!(!ap1.is_divergent_with(&ap2));
    }

    #[test]
    fn test_is_divergent_with_extra_asn_in_middle() {
        let ap1 = AsPath::new(vec![Asn::new(1), Asn::new(2), Asn::new(3)], Vec::new());
        let ap2 = AsPath::new(
            vec![Asn::new(4), Asn::new(2), Asn::new(5), Asn::new(3)],
            Vec::new(),
        );

        assert!(ap1.is_divergent_with(&ap2));
    }

    #[test]
    fn test_is_divergent_with_different_path_after_shared_asn() {
        let ap1 = AsPath::new(
            vec![Asn::new(1), Asn::new(2), Asn::new(5), Asn::new(3)],
            Vec::new(),
        );
        let ap2 = AsPath::new(
            vec![Asn::new(4), Asn::new(2), Asn::new(6), Asn::new(3)],
            Vec::new(),
        );

        assert!(ap1.is_divergent_with(&ap2));
    }

    #[test]
    fn test_is_divergent_with_missing_asns() {
        let ap1 = AsPath::new(
            vec![Asn::new(1), Asn::new(2), Asn::new(5), Asn::new(3)],
            Vec::new(),
        );
        let ap2 = AsPath::new(vec![Asn::new(4), Asn::new(2), Asn::new(3)], Vec::new());

        assert!(ap1.is_divergent_with(&ap2));
    }

    #[test]
    fn test_is_divergent_with_no_shared_asns_except_origin() {
        let ap1 = AsPath::new(vec![Asn::new(1), Asn::new(2), Asn::new(5)], Vec::new());
        let ap2 = AsPath::new(vec![Asn::new(3), Asn::new(4), Asn::new(5)], Vec::new());

        assert!(!ap1.is_divergent_with(&ap2));
    }

    #[test]
    fn test_is_divergent_with_different_origins_panics() {
        assert!(
            std::panic::catch_unwind(|| {
                let ap1 = AsPath::new(vec![Asn::new(1), Asn::new(2), Asn::new(3)], Vec::new());
                let ap2 = AsPath::new(vec![Asn::new(1), Asn::new(2), Asn::new(4)], Vec::new());
                ap1.is_divergent_with(&ap2);
            })
            .is_err()
        );
    }

    #[test]
    fn test_serialize() {
        let mut ap = AsPath::new(vec![Asn::new(1), Asn::new(2), Asn::new(3)], Vec::new());
        let route = Route::get_mock(Some(ap.clone()));
        ap.add_route(route.clone());

        let json = serde_json::to_string(&ap).unwrap();
        assert!(
            json == "{\"as_path\":[1,2,3],\"routes\":".to_owned()
                + serde_json::to_string(&vec![route]).unwrap().as_str()
                + "}"
        );
    }

    #[test]
    fn test_as_path_eq() {
        // EQ with same default origin and default AS path
        let mut ap_1 = AsPath::get_mock(None, None);
        ap_1.add_route(Route::get_mock(None));
        let mut ap_2 = AsPath::get_mock(None, None);
        ap_2.add_route(Route::get_mock(None));
        assert_eq!(ap_1, ap_2);

        // EQ with same explicit origin and explicit AS path
        ap_1 = AsPath::get_mock(Some(Vec::from([Asn::new(1)])), None);
        ap_1.add_route(Route::get_mock(Some(ap_1.clone())));
        ap_2 = AsPath::get_mock(Some(Vec::from([Asn::new(1)])), None);
        ap_2.add_route(Route::get_mock(Some(ap_2.clone())));
        assert_eq!(ap_1, ap_2);
    }

    #[test]
    fn test_as_path_ne() {
        // NE with different origins
        let mut ap_1 = AsPath::get_mock(Some(Vec::from([Asn::new(1)])), None);
        let mut ap_2 = AsPath::get_mock(Some(Vec::from([Asn::new(2)])), None);
        assert_ne!(ap_1, ap_2);

        // NE if missing Route
        ap_1 = AsPath::get_mock(Some(Vec::from([Asn::new(1)])), None);
        ap_1.add_route(Route::get_mock(Some(ap_1.clone())));
        ap_2 = AsPath::get_mock(Some(Vec::from([Asn::new(1)])), None);
        assert_ne!(ap_1, ap_2);

        // NE if different routes
        ap_1 = AsPath::get_mock(Some(Vec::from([Asn::new(1)])), None);
        ap_1.add_route(Route::get_mock(Some(ap_1.clone())));
        ap_2 = AsPath::get_mock(Some(Vec::from([Asn::new(1)])), None);
        ap_1.add_route(Route::get_mock(Some(AsPath::get_mock(
            Some(Vec::from([Asn::new(2)])),
            None,
        ))));
        assert_ne!(ap_1, ap_2);
    }
}
