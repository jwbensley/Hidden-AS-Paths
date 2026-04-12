use crate::types::as_path::AsPath;
use crate::types::asn::Asn;
use crate::types::route::Route;
use log::debug;
use serde::Serialize;
use std::collections::HashSet;
use std::collections::hash_set::Drain;

/// A set of AS paths which all point to the same origin ASN
#[derive(Debug, Clone, Serialize, Default)]
pub struct OriginAsPaths {
    origin: Asn,
    as_paths: HashSet<AsPath>,
    diverging_asns: HashSet<Asn>,
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
        OriginAsPaths {
            origin,
            as_paths,
            diverging_asns: HashSet::new(),
        }
    }

    pub fn get_mock(origin: Option<Asn>, as_path: Option<AsPath>) -> OriginAsPaths {
        let o = origin.unwrap_or(Asn::get_mock(None));
        let a = as_path.unwrap_or(AsPath::get_mock(None, None));
        OriginAsPaths::new(o.clone(), HashSet::from([a]))
    }

    pub fn get_as_paths(&self) -> &HashSet<AsPath> {
        &self.as_paths
    }

    pub fn has_as_path(&self, new_path: &AsPath) -> bool {
        for existing_path in self.get_as_paths() {
            if existing_path.get_asns() == new_path.get_asns() {
                debug!("AS path found: {:#?}", new_path);
                return true;
            }
        }
        debug!("AS path not found: {:#?}", new_path);
        false
    }

    pub fn get_origin(&self) -> &Asn {
        &self.origin
    }

    pub fn has_route(&self, route: &Route) -> bool {
        if route.get_origin() != self.get_origin() {
            panic!(
                "Can't check route for different origin: {} != {:#?}",
                self.get_origin(),
                route
            )
        };
        for as_path in self.get_as_paths() {
            if as_path.has_route(route) {
                return true;
            }
        }
        false
    }

    pub fn add_as_path(&mut self, as_path: AsPath) {
        if !self.has_as_path(&as_path) {
            debug!("Adding new AS path: {:#?}", as_path);
            self.as_paths.insert(as_path);
        }
    }

    pub fn get_as_paths_mut(&mut self) -> HashSet<AsPath> {
        std::mem::take(&mut self.as_paths)
    }

    pub fn pop_as_paths(&mut self) -> Drain<'_, AsPath> {
        self.as_paths.drain()
    }

    pub fn add_route(&mut self, route: Route, as_path: &AsPath) {
        if self.has_route(&route) {
            return;
        };

        if !self.has_as_path(as_path) {
            self.add_as_path(as_path.clone());
        }

        let mut existing = self
            .as_paths
            .take(as_path)
            .unwrap_or_else(|| panic!("AS Path {:#?} not found in {:#?}", as_path, self.as_paths));
        existing.add_route(route);
        self.as_paths.insert(existing);
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        self.as_paths.len()
    }

    pub fn get_diverging_asns(&self) -> &HashSet<Asn> {
        &self.diverging_asns
    }

    fn remove_as_path(&mut self, as_path: &AsPath) {
        self.as_paths.remove(as_path);
    }

    pub fn remove_single_hop_paths(&mut self) {
        let mut to_remove = Vec::new();
        for as_path in self.get_as_paths() {
            // We may also see zero length AS paths, for iBGP originated prefixes,
            // announced to a public collector using iBGP.
            if as_path.len() <= 1 {
                to_remove.push(as_path.clone());
            }
        }
        debug!("Single-hop AS Paths to remove: {}", to_remove.len());
        for as_path in to_remove {
            self.remove_as_path(&as_path);
        }
    }

    fn get_non_divergent_paths(&self) -> Vec<&AsPath> {
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
                    non_divergent_paths.retain(|x| *x != a && *x != b);
                }
            }
            checked.push(a);
        }
        non_divergent_paths
    }

    pub fn remove_non_divergent_as_paths(&mut self) {
        let mut as_paths = Vec::new();
        for as_path in self.get_non_divergent_paths() {
            as_paths.push(as_path.clone());
        }
        for as_path in as_paths {
            self.remove_as_path(&as_path);
        }
    }

    pub fn populate_diverging_asns(&mut self) {
        let mut checked = Vec::<&AsPath>::new();
        let mut diverging_asns = Vec::new();

        for a in self.get_as_paths() {
            for b in self.get_as_paths() {
                if a == b {
                    continue;
                };
                if checked.contains(&b) {
                    continue;
                }
                if a.len() == 1 || b.len() == 1 {
                    continue;
                }
                if a.is_divergent_with(b) {
                    diverging_asns.push(a.get_diverging_asn(b).clone());
                }
            }
            checked.push(a);
        }
        diverging_asns.iter().for_each(|value| {
            self.diverging_asns.insert(value.clone());
        });
    }

    pub fn remove_as_paths_with_only_known_community_asns(&mut self, known_asns: &[Asn]) {
        let mut to_remove = Vec::new();
        for as_path in self.get_as_paths() {
            if !as_path.has_unknown_community_asns(known_asns) {
                to_remove.push(as_path.clone());
            }
        }
        debug!(
            "AS Paths with unknown communities to remove: {}",
            to_remove.len()
        );
        for as_path in to_remove {
            self.remove_as_path(&as_path);
        }
    }

    pub fn remove_communities_with_known_asns(&mut self, known_asns: &[Asn]) {
        let mut updated_paths = HashSet::with_capacity(self.len());
        for mut as_path in self.get_as_paths_mut() {
            as_path.remove_communities_with_known_asns(known_asns);
            updated_paths.insert(as_path);
        }
        self.as_paths = updated_paths;
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
        let invalid_as_path = AsPath::new(vec![Asn::new(2)], vec![]);
        let as_paths = HashSet::from([invalid_as_path]);
        let result = std::panic::catch_unwind(|| OriginAsPaths::new(origin.clone(), as_paths));
        assert!(result.is_err());
    }

    #[test]
    fn test_get_mock() {
        let oap = OriginAsPaths::get_mock(None, None);
        assert_eq!(oap.get_origin(), &Asn::get_mock(None));
        assert_eq!(
            oap.get_as_paths(),
            &HashSet::from([AsPath::get_mock(None, None)])
        );
    }

    #[test]
    fn test_get_mock_with_origin() {
        let origin = Asn::new(42);
        let oap = OriginAsPaths::get_mock(
            Some(origin.clone()),
            Some(AsPath::new(vec![origin.clone()], vec![])),
        );
        assert_eq!(oap.get_origin(), &origin);
        assert_eq!(
            oap.get_as_paths(),
            &HashSet::from([AsPath::new(vec![origin.clone()], vec![])]),
        );
    }

    #[test]
    fn test_get_as_paths() {
        let oap = OriginAsPaths::get_mock(None, None);
        assert_eq!(
            oap.get_as_paths(),
            &HashSet::from([AsPath::get_mock(None, None)])
        );
    }

    #[test]
    fn test_has_as_path() {
        let mut oap = OriginAsPaths::get_mock(None, None);
        let as_path = AsPath::get_mock(None, None);
        assert!(oap.has_as_path(&as_path));

        let new_path = AsPath::new(vec![Asn::new(999)], vec![]);
        assert!(!oap.has_as_path(&new_path));

        oap.add_as_path(new_path.clone());
        assert!(oap.has_as_path(&new_path));
    }

    #[test]
    fn test_get_origin() {
        let origin = Asn::new(42);
        let oap = OriginAsPaths::new(origin.clone(), HashSet::new());
        assert_eq!(oap.get_origin(), &origin);
    }

    #[test]
    fn test_has_route_missing() {
        let oap = OriginAsPaths::get_mock(None, None);
        let route = Route::get_mock(None);
        assert!(!oap.has_route(&route));
    }

    #[test]
    fn test_has_route() {
        let as_path: AsPath = AsPath::get_mock(None, None);
        let mut oap =
            OriginAsPaths::get_mock(Some(as_path.get_origin().clone()), Some(as_path.clone()));
        let route = Route::get_mock(None);
        oap.add_route(route.clone(), &as_path);
        assert!(oap.has_route(&route));
    }

    #[test]
    fn test_add_as_path() {
        let mut oap = OriginAsPaths::get_mock(None, None);
        let initial_len = oap.len();

        let new_path = AsPath::new(vec![Asn::new(100), Asn::new(200)], vec![]);
        oap.add_as_path(new_path.clone());
        assert_eq!(oap.len(), initial_len + 1);
        assert!(oap.has_as_path(&new_path));

        // Adding the same path again should not increase length
        oap.add_as_path(new_path.clone());
        assert_eq!(oap.len(), initial_len + 1);
    }

    #[test]
    fn test_add_route() {
        let as_path: AsPath = AsPath::get_mock(None, None);
        let mut oap =
            OriginAsPaths::get_mock(Some(as_path.get_origin().clone()), Some(as_path.clone()));
        let route = Route::get_mock(Some(as_path.get_origin().clone()));

        assert!(!oap.has_route(&route));
        oap.add_route(route.clone(), &as_path);
        assert!(oap.has_route(&route));
    }

    #[test]
    fn test_add_route_existing() {
        let as_path: AsPath = AsPath::get_mock(None, None);
        let mut oap =
            OriginAsPaths::get_mock(Some(as_path.get_origin().clone()), Some(as_path.clone()));
        let route = Route::get_mock(Some(as_path.get_origin().clone()));

        oap.add_route(route.clone(), &as_path);
        assert!(oap.has_route(&route));

        let mut total = 0;
        for asp in oap.get_as_paths() {
            total += asp.get_routes().len();
        }
        assert_eq!(total, 1);

        oap.add_route(route.clone(), &as_path);
        assert!(oap.has_route(&route));

        total = 0;
        for asp in oap.get_as_paths() {
            total += asp.get_routes().len();
        }
        assert_eq!(total, 1);
    }

    #[test]
    fn test_is_empty() {
        let empty_oap = OriginAsPaths::new(Asn::new(100), HashSet::new());
        assert!(empty_oap.is_empty());

        let non_empty_oap = OriginAsPaths::get_mock(None, None);
        assert!(!non_empty_oap.is_empty());
    }

    #[test]
    fn test_len() {
        let oap = OriginAsPaths::new(Asn::new(100), HashSet::new());
        assert_eq!(oap.len(), 0);

        let oap = OriginAsPaths::get_mock(None, None);
        assert_eq!(oap.len(), 1);

        let oap = OriginAsPaths::new(
            Asn::new(1),
            HashSet::from([
                AsPath::get_mock(None, None),
                AsPath::get_mock(Some(vec![Asn::new(1)]), None),
            ]),
        );
        assert_eq!(oap.len(), 2);
    }

    #[test]
    fn test_remove_as_path() {
        let path_1 = AsPath::get_mock(None, None);
        let path_2 = AsPath::new(vec![Asn::new(100), Asn::get_mock(None)], vec![]);
        let mut oap = OriginAsPaths::new(
            Asn::get_mock(None),
            HashSet::from([path_1.clone(), path_2.clone()]),
        );
        assert_eq!(oap.len(), 2);
        oap.remove_as_path(&path_1);
        assert_eq!(oap.len(), 1);
        assert!(!oap.has_as_path(&path_1));
        assert!(oap.has_as_path(&path_2));
    }

    #[test]
    fn test_remove_as_path_missing() {
        let mut oap = OriginAsPaths::new(
            Asn::get_mock(None),
            HashSet::from([AsPath::get_mock(None, None)]),
        );
        assert_eq!(oap.len(), 1);
        let missing_path = AsPath::new(vec![Asn::new(999)], vec![]);
        oap.remove_as_path(&missing_path);
        assert_eq!(oap.len(), 1);
        assert!(oap.has_as_path(&AsPath::get_mock(None, None)));
    }

    #[test]
    fn test_remove_single_hop_paths() {
        let mut oap = OriginAsPaths::new(
            Asn::new(100),
            HashSet::from([AsPath::new(vec![Asn::new(100)], vec![])]),
        );
        assert_eq!(oap.len(), 1);
        oap.remove_single_hop_paths();
        assert_eq!(oap.len(), 0);
    }

    #[test]
    fn test_remove_single_hop_paths_mixed() {
        let path_1 = AsPath::new(vec![Asn::new(200)], vec![]);
        let path_2 = AsPath::new(vec![Asn::new(100), Asn::new(200)], vec![]);
        let mut oap = OriginAsPaths::new(
            Asn::new(200),
            HashSet::from([path_1.clone(), path_2.clone()]),
        );
        assert_eq!(oap.len(), 2);
        oap.remove_single_hop_paths();
        assert_eq!(oap.len(), 1);
        assert!(!oap.has_as_path(&path_1));
        assert!(oap.has_as_path(&path_2));
    }

    #[test]
    fn test_get_non_divergent_paths_empty() {
        let oap = OriginAsPaths::new(Asn::new(100), HashSet::new());
        let non_divergent = oap.get_non_divergent_paths();
        assert!(non_divergent.is_empty());
    }

    #[test]
    fn test_get_non_divergent_paths_single_path() {
        let oap = OriginAsPaths::get_mock(None, None);
        let non_divergent = oap.get_non_divergent_paths();
        assert!(!non_divergent.is_empty());
    }

    #[test]
    fn test_get_non_divergent_paths_non_divergent_paths() {
        let path_1 = AsPath::new(vec![Asn::new(100)], vec![]);
        let path_2 = AsPath::new(vec![Asn::new(200), Asn::new(100)], vec![]);
        let path_3 = AsPath::new(vec![Asn::new(300), Asn::new(200), Asn::new(100)], vec![]);
        let oap = OriginAsPaths::new(
            Asn::new(100),
            HashSet::from([path_1.clone(), path_2.clone(), path_3.clone()]),
        );
        let non_divergent = oap.get_non_divergent_paths();
        assert_eq!(non_divergent.len(), 3);
        assert!(non_divergent.contains(&&path_1));
        assert!(non_divergent.contains(&&path_2));
        assert!(non_divergent.contains(&&path_3));
    }

    #[test]
    fn test_get_non_divergent_paths_divergent_paths() {
        let path_1 = AsPath::new(vec![Asn::new(300), Asn::new(200), Asn::new(100)], vec![]);
        let path_2 = AsPath::new(vec![Asn::new(300), Asn::new(400), Asn::new(100)], vec![]);
        let path_3 = AsPath::new(vec![Asn::new(300), Asn::new(100)], vec![]);
        let oap = OriginAsPaths::new(
            Asn::new(100),
            HashSet::from([path_1.clone(), path_2.clone(), path_3.clone()]),
        );
        let non_divergent = oap.get_non_divergent_paths();
        assert_eq!(non_divergent.len(), 0);
        assert!(!non_divergent.contains(&&path_1));
        assert!(!non_divergent.contains(&&path_2));
        assert!(!non_divergent.contains(&&path_3));
    }

    #[test]
    fn test_get_non_divergent_paths_mixed() {
        let path_1 = AsPath::new(vec![Asn::new(300), Asn::new(200), Asn::new(100)], vec![]);
        let path_2 = AsPath::new(vec![Asn::new(600), Asn::new(400), Asn::new(100)], vec![]);
        let path_3 = AsPath::new(vec![Asn::new(300), Asn::new(100)], vec![]);
        let oap = OriginAsPaths::new(
            Asn::new(100),
            HashSet::from([path_1.clone(), path_2.clone(), path_3.clone()]),
        );
        let non_divergent = oap.get_non_divergent_paths();
        assert_eq!(non_divergent.len(), 1);
        assert!(!non_divergent.contains(&&path_1));
        assert!(non_divergent.contains(&&path_2));
        assert!(!non_divergent.contains(&&path_3));
    }

    #[test]
    fn test_origin_as_paths_eq() {
        let oap_1 = OriginAsPaths::get_mock(None, None);
        let oap_2 = OriginAsPaths::get_mock(None, None);
        assert_eq!(oap_1, oap_2);

        let oap_1 = OriginAsPaths::get_mock(Some(Asn::new(1)), None);
        let oap_2 = OriginAsPaths::get_mock(Some(Asn::new(1)), None);
        assert_eq!(oap_1, oap_2);
    }

    #[test]
    fn test_origin_as_paths_ne() {
        let oap_1 = OriginAsPaths::new(Asn::new(1), HashSet::new());
        let oap_2 = OriginAsPaths::new(Asn::new(2), HashSet::new());
        assert_ne!(oap_1, oap_2);
    }
}
