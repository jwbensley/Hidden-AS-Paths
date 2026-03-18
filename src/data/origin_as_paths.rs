use crate::types::as_path::AsPath;
use crate::types::asn::Asn;
use crate::types::route::Route;
use log::debug;
use serde::Serialize;

/// A vector of AS paths which all point to the same origin ASN
#[derive(Debug, Clone, Serialize)]
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
    pub fn new(origin: Asn, as_paths: Vec<AsPath>) -> Self {
        OriginAsPaths { origin, as_paths }
    }

    pub fn get_mock(origin: Option<Asn>) -> OriginAsPaths {
        let o = origin.unwrap_or(Asn::get_mock(None));
        OriginAsPaths::new(
            o.clone(),
            Vec::from([AsPath::get_mock(Some(Vec::from([o])))]),
        )
    }

    fn get_as_paths(&self) -> &Vec<AsPath> {
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
            self.as_paths.push(as_path);
        };
    }

    fn get_as_paths_mut(&mut self) -> &mut Vec<AsPath> {
        self.as_paths.as_mut()
    }

    fn get_as_path_mut(&mut self, as_path: &AsPath) -> &mut AsPath {
        for a in self.get_as_paths_mut() {
            if a.get_asns() == as_path.get_asns() {
                return a;
            }
        }
        panic!("AS Path not found {:#?}", as_path);
    }

    pub fn add_route(&mut self, route: Route) {
        let as_path = route.get_as_path().clone();
        self.get_as_path_mut(&as_path).add_route(route);
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        self.as_paths.len()
    }

    fn remove_as_path(&mut self, as_path: &AsPath) {
        self.as_paths
            .remove(self.as_paths.iter().position(|x| x == as_path).unwrap());
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
