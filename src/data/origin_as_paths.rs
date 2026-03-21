use crate::types::as_path::AsPath;
use crate::types::asn::Asn;
use crate::types::route::Route;
use log::debug;
use serde::Serialize;
use std::collections::HashSet;

/// A set of AS paths which all point to the same origin ASN
#[derive(Debug, Clone, Serialize)]
pub struct OriginAsPaths {
    origin: Asn,
    as_paths: HashSet<AsPath>,
}

impl PartialEq for OriginAsPaths {
    fn eq(&self, other: &Self) -> bool {
        (self.as_paths == other.as_paths) && (self.origin == other.origin)
    }
}

impl OriginAsPaths {
    pub fn new(origin: Asn, as_paths: HashSet<AsPath>) -> Self {
        for as_path in &as_paths {
            if as_path.get_origin() != &origin {
                panic!(
                    "Path origin {} does not match expected origin {}",
                    as_path.get_origin(),
                    origin
                );
            }
        }
        OriginAsPaths { origin, as_paths }
    }

    pub fn get_mock(origin: Option<Asn>) -> OriginAsPaths {
        let o = origin.unwrap_or(Asn::get_mock(None));
        OriginAsPaths::new(
            o.clone(),
            HashSet::from([AsPath::get_mock(Some(vec![o.clone()]), None)]),
        )
    }

    pub fn get_as_paths(&self) -> &HashSet<AsPath> {
        &self.as_paths
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

    pub fn get_origin(&self) -> &Asn {
        &self.origin
    }

    fn get_as_path(&self, as_path: &AsPath) -> &AsPath {
        for a in self.get_as_paths() {
            if a.get_asns() == as_path.get_asns() {
                return a;
            }
        }
        panic!("AS Path not found {:#?}", as_path);
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

    pub fn add_as_path(&mut self, as_path: AsPath) {
        if !self.has_as_path(&as_path) {
            self.as_paths.insert(as_path);
        };
    }

    // fn get_as_paths_mut(&mut self) -> &mut HashSet<AsPath> {
    //     &mut self.as_paths
    // }

    // fn get_as_path_mut(&mut self, as_path: &AsPath) -> &mut AsPath {
    //     for a in self.get_as_paths_mut().iter() {
    //         if a.get_asns() == as_path.get_asns() {
    //             return &mut self.get_as_paths_mut().take(a).unwrap();
    //         }
    //     }
    //     panic!("AS Path not found {:#?}", as_path);
    // }

    pub fn add_route(&mut self, route: Route) {
        let as_path = route.get_as_path().clone();
        let mut existing = self
            .as_paths
            .take(&as_path)
            .unwrap_or_else(|| panic!("AS Path not found {:#?}", as_path));
        existing.add_route(route);
        self.as_paths.insert(existing);
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        self.as_paths.len()
    }

    fn remove_as_path(&mut self, as_path: &AsPath) {
        self.as_paths.remove(as_path);
    }

    pub fn remove_single_hop_paths(&mut self) {
        let mut to_remove = Vec::new();
        for as_path in self.get_as_paths() {
            // We may also see zero length AS paths, for iBGP originated prefixes,
            //announced to a public collector using iBGP.
            if as_path.len() <= 1 {
                to_remove.push(as_path.clone());
            }
        }
        debug!("Single-hop AS Paths to remove: {}", to_remove.len());
        for as_path in to_remove {
            self.remove_as_path(&as_path);
        }
    }

    pub fn find_non_divergent_paths(&self) -> Vec<&AsPath> {
        let mut non_divergent_paths: Vec<&AsPath> = self.get_as_paths().iter().collect();
        let mut checked = Vec::<&AsPath>::new();

        for a in self.get_as_paths() {
            for b in self.get_as_paths() {
                if a == b {
                    continue;
                };
                if checked.contains(&b) {
                    continue;
                }
                if a.is_divergent_with(b) {
                    non_divergent_paths.retain(|x| *x != b);
                }
            }
            checked.push(a);
        }
        non_divergent_paths
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let test_values = vec![
            (
                Asn::get_mock(None),
                HashSet::from([AsPath::get_mock(None, None)]),
            ),
            (Asn::new(1), HashSet::from([])),
        ];
        for (origin, as_paths) in test_values {
            let oap = OriginAsPaths::new(origin.clone(), as_paths.clone());
            assert_eq!(oap.get_origin(), &origin);
            assert_eq!(oap.get_as_paths(), &as_paths);
        }
    }

    #[test]
    fn test_new_invalid() {
        let origin = Asn::new(1);
        let invalid_as_path = AsPath::new(vec![Asn::new(2)], Vec::new());
        let as_paths = HashSet::from([invalid_as_path]);
        let result = std::panic::catch_unwind(|| OriginAsPaths::new(origin.clone(), as_paths));
        assert!(result.is_err());
    }

    #[test]
    fn test_get_mock() {
        let oap = OriginAsPaths::get_mock(None);
        assert_eq!(oap.get_origin(), &Asn::get_mock(None));
        assert_eq!(
            oap.get_as_paths(),
            &HashSet::from([AsPath::get_mock(None, None)])
        );
    }

    #[test]
    fn test_get_mock_with_origin() {
        let origin = Asn::new(42);
        let oap = OriginAsPaths::get_mock(Some(origin.clone()));
        assert_eq!(oap.get_origin(), &origin);
        assert_eq!(
            oap.get_as_paths(),
            &HashSet::from([AsPath::get_mock(Some(vec![origin.clone()]), None)])
        );
    }

    #[test]
    fn test_get_as_paths() {
        let oap = OriginAsPaths::get_mock(None);
        assert_eq!(
            oap.get_as_paths(),
            &HashSet::from([AsPath::get_mock(None, None)])
        );
    }

    #[test]
    fn test_has_as_path() {
        let mut oap = OriginAsPaths::get_mock(None);
        let as_path = AsPath::get_mock(None, None);
        assert!(oap.has_as_path(&as_path));

        let new_path = AsPath::get_mock(Some(vec![Asn::new(999)]), None);
        assert!(!oap.has_as_path(&new_path));

        oap.add_as_path(new_path.clone());
        assert!(oap.has_as_path(&new_path));
    }

    #[test]
    fn test_get_origin() {
        let origin = Asn::new(42);
        let oap = OriginAsPaths::get_mock(Some(origin.clone()));
        assert_eq!(oap.get_origin(), &origin);
    }

    #[test]
    fn test_get_as_path() {
        let oap = OriginAsPaths::get_mock(None);
        let as_path = AsPath::get_mock(None, None);
        assert_eq!(oap.get_as_path(&as_path), &as_path);
    }

    #[test]
    fn test_get_as_path_missing() {
        assert!(
            std::panic::catch_unwind(|| {
                let oap = OriginAsPaths::get_mock(None);
                let as_path = AsPath::get_mock(Some(vec![Asn::new(999)]), None);
                oap.get_as_path(&as_path);
            })
            .is_err()
        );
    }

    #[test]
    fn test_has_route_wrong_origin() {
        assert!(
            std::panic::catch_unwind(|| {
                let oap = OriginAsPaths::get_mock(None);
                let route =
                    Route::get_mock(Some(AsPath::get_mock(Some(vec![Asn::new(999)]), None)));
                oap.has_route(&route);
            })
            .is_err()
        );
    }

    #[test]
    fn test_has_route_missing_as_path() {
        let oap = OriginAsPaths::get_mock(None);
        let route = Route::get_mock(Some(AsPath::get_mock(Some(vec![Asn::new(999)]), None)));
        assert!(!oap.has_route(&route));
    }

    #[test]
    fn test_has_route() {
        let oap = OriginAsPaths::get_mock(None);
        let route = Route::get_mock(None);
        assert!(oap.has_route(&route));
    }

    #[test]
    fn test_add_as_path() {
        let mut oap = OriginAsPaths::get_mock(None);
        let initial_len = oap.len();

        let new_path = AsPath::get_mock(Some(vec![Asn::new(100), Asn::new(200)]), None);
        oap.add_as_path(new_path.clone());
        assert_eq!(oap.len(), initial_len + 1);
        assert!(oap.has_as_path(&new_path));

        // Adding the same path again should not increase length
        oap.add_as_path(new_path.clone());
        assert_eq!(oap.len(), initial_len + 1);
    }

    #[test]
    fn test_add_route() {
        let mut oap = OriginAsPaths::get_mock(None);
        assert_eq!(oap.len(), 1);

        let route = Route::get_mock(Some(AsPath::get_mock(Some(vec![Asn::new(999)]), None)));
        assert!(!oap.has_route(&route));
        oap.add_route(route.clone());
        assert_eq!(oap.len(), 2);
        assert!(oap.has_route(&route));
    }

    #[test]
    fn test_add_route_missing_as_path() {
        assert!(
            std::panic::catch_unwind(|| {
                let mut oap = OriginAsPaths::get_mock(None);
                assert_eq!(oap.len(), 1);
                let route =
                    Route::get_mock(Some(AsPath::get_mock(Some(vec![Asn::new(999)]), None)));
                oap.add_route(route);
            })
            .is_err()
        );
    }

    #[test]
    fn test_is_empty() {
        let empty_oap = OriginAsPaths::new(Asn::new(100), HashSet::new());
        assert!(empty_oap.is_empty());

        let non_empty_oap = OriginAsPaths::get_mock(None);
        assert!(!non_empty_oap.is_empty());
    }

    #[test]
    fn test_len() {
        let oap = OriginAsPaths::new(Asn::new(100), HashSet::new());
        assert_eq!(oap.len(), 0);

        let oap = OriginAsPaths::get_mock(None);
        assert_eq!(oap.len(), 1);

        let oap = OriginAsPaths::new(
            Asn::new(100),
            HashSet::from([
                AsPath::get_mock(None, None),
                AsPath::get_mock(Some(vec![Asn::new(1)]), None),
            ]),
        );
        assert_eq!(oap.len(), 2);
    }

    #[test]
    fn test_remove_single_hop_paths() {
        let mut oap = OriginAsPaths::new(
            Asn::new(100),
            HashSet::from([
                AsPath::get_mock(Some(vec![Asn::new(1)]), None),
                AsPath::get_mock(Some(vec![Asn::new(1), Asn::new(2)]), None),
                AsPath::get_mock(Some(vec![Asn::new(1), Asn::new(2), Asn::new(3)]), None),
            ]),
        );

        oap.remove_single_hop_paths();
        assert_eq!(oap.len(), 2);

        for path in oap.get_as_paths() {
            assert!(path.len() > 1);
        }
    }

    #[test]
    fn test_find_non_divergent_paths() {
        let oap = OriginAsPaths::get_mock(None);
        let non_divergent = oap.find_non_divergent_paths();
        assert!(!non_divergent.is_empty());
    }

    #[test]
    fn test_origin_as_paths_eq() {
        let oap_1 = OriginAsPaths::get_mock(None);
        let oap_2 = OriginAsPaths::get_mock(None);
        assert_eq!(oap_1, oap_2);

        let oap_1 = OriginAsPaths::get_mock(Some(Asn::new(1)));
        let oap_2 = OriginAsPaths::get_mock(Some(Asn::new(1)));
        assert_eq!(oap_1, oap_2);
    }

    #[test]
    fn test_origin_as_paths_ne() {
        let oap_1 = OriginAsPaths::get_mock(Some(Asn::new(1)));
        let oap_2 = OriginAsPaths::get_mock(Some(Asn::new(2)));
        assert_ne!(oap_1, oap_2);
    }
}
