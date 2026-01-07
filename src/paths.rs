pub mod as_paths {
    use bgpkit_parser::models::{Asn, Community, LargeCommunity, Peer};
    use ipnet::IpNet;
    use log::debug;
    use std::net::IpAddr;

    /// Store a route pulled from an MRT file (one route object per prefix)
    #[derive(Clone, Debug)]
    pub struct Route {
        as_path: Vec<Asn>,
        filename: String,
        next_hop: IpAddr,
        peer: Peer,
        prefix: IpNet,
        communities: Vec<Community>,
        large_communities: Vec<LargeCommunity>,
    }

    impl PartialEq for Route {
        fn eq(&self, other: &Self) -> bool {
            (self.as_path == other.as_path)
                && (self.filename == other.filename)
                && (self.next_hop == other.next_hop)
                && (self.peer == other.peer)
                && (self.prefix == other.prefix)
        }
    }

    impl Route {
        pub fn new(
            as_path: Vec<Asn>,
            filename: String,
            next_hop: IpAddr,
            peer: Peer,
            prefix: IpNet,
            communities: Vec<Community>,
            large_communities: Vec<LargeCommunity>,
        ) -> Self {
            Self {
                as_path,
                filename,
                next_hop,
                peer,
                prefix,
                communities,
                large_communities,
            }
        }

        pub fn get_as_path(&self) -> &Vec<Asn> {
            &self.as_path
        }

        pub fn get_communities(&self) -> &Vec<Community> {
            &self.communities
        }

        pub fn get_large_communities(&self) -> &Vec<LargeCommunity> {
            &self.large_communities
        }

        pub fn get_origin(&self) -> &Asn {
            self.as_path.last().unwrap()
        }
    }

    /// A deduped AS path which stores one or more routes
    #[derive(Debug, Clone)]
    pub struct AsPath {
        as_path: Vec<Asn>,
        routes: Vec<Route>,
    }

    // impl Default for AsPath {
    //     fn default() -> Self {
    //         Self::new(Vec::new())
    //     }
    // }

    impl PartialEq for AsPath {
        fn eq(&self, other: &Self) -> bool {
            (self.routes == other.routes) && (self.as_path == other.as_path)
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

        pub fn add_route(&mut self, route: Route) {
            if self.has_route(&route) {
                return;
            };
            self.routes.push(route);
        }

        // pub fn as_path_len(&self) -> usize {
        //     self.as_path.len()
        // }

        pub fn get_as_path(&self) -> &Vec<Asn> {
            &self.as_path
        }

        // pub fn get_origin(&self) -> &Asn {
        //     &self.get_as_path().last().unwrap()
        // }

        // pub fn get_routes(&self) -> &Vec<Route> {
        //     &self.routes
        // }

        pub fn has_route(&self, route: &Route) -> bool {
            let present = self.routes.contains(route);
            debug!("Route present {:#?}: {}", route, present);
            present
        }

        // pub fn has_routes(&self) -> bool {
        //     self.route_count() > 0
        // }

        // pub fn insert_route(&mut self, route: Route) {
        //     debug!("Adding route: {:#?}", route);
        //     if !self.has_route(&route) {
        //         self.routes.push(route);
        //     }
        // }

        // pub fn route_count(&self) -> usize {
        //     self.routes.len()
        // }
    }

    /// A vector of unique, deduped AS paths, which all point to the same origin ASN
    #[derive(Debug, Clone)]
    pub struct OriginAsPaths {
        origin: Asn,
        as_paths: Vec<AsPath>,
    }

    // impl Default for OriginAsPaths {
    //     fn default() -> Self {
    //         Self::new()
    //     }
    // }

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

        pub fn merge_from(&mut self, other: &Self) {
            for as_path in other.get_as_paths() {
                self.add_as_path(as_path.clone());
            }
        }
    }
}
