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

        // pub fn get_communities(&self) -> &Vec<Community> {
        //     &self.communities
        // }

        // pub fn get_large_communities(&self) -> &Vec<LargeCommunity> {
        //     &self.large_communities
        // }

        pub fn get_origin(&self) -> &Asn {
            &self.as_path.last().unwrap()
        }
    }

    /// A deduped AS path which stores one or more routes
    #[derive(Debug)]
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
                as_path: as_path,
                routes: Vec::<Route>::new(),
            }
        }

        pub fn add_route(&mut self, route: Route) {
            if self.has_route(&route) {return; };
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
            debug!("Route present {:?}: {}", route, present);
            present
        }

        // pub fn has_routes(&self) -> bool {
        //     self.route_count() > 0
        // }

        // pub fn insert_route(&mut self, route: Route) {
        //     debug!("Adding route: {:?}", route);
        //     if !self.has_route(&route) {
        //         self.routes.push(route);
        //     }
        // }

        // pub fn route_count(&self) -> usize {
        //     self.routes.len()
        // }

    }

    /// A vector of unique, deduped AS paths, which all point to the same origin ASN
    #[derive(Debug)]
    pub struct OriginAsPaths {
        origin: Asn,
        as_paths: Vec<AsPath>
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
        // pub fn as_path_count(&self) -> usize {
        //     self.as_paths.len()
        // }

        pub fn new(origin: Asn) -> Self {
            OriginAsPaths {
                origin: origin,
                as_paths: Vec::<AsPath>::new(),
            }
        }

        pub fn add_as_path(&mut self, as_path: Vec<Asn>) {
            if self.has_as_path(&as_path) { return };
            self.as_paths.push(AsPath::new(as_path));
        }

        pub fn add_route(&mut self, route: Route) {
            if !self.has_as_path(&route.get_as_path()) { self.add_as_path(route.get_as_path().clone()); };
            self.get_as_path_mut(&route.get_as_path()).add_route(route);
        }

        fn get_as_paths(&self) -> &Vec<AsPath> {
            &self.as_paths
        }

        fn get_as_paths_mut(&mut self) -> &mut Vec<AsPath> {
            self.as_paths.as_mut()
        }

        fn get_as_path(&self, as_path: &Vec<Asn>) -> &AsPath {
            for a in self.get_as_paths() {
                if a.get_as_path() == as_path {
                    return a;
                }
            }
            panic!("AS Path not found {:?}", as_path);
        }

        fn get_as_path_mut(&mut self, as_path: &Vec<Asn>) -> &mut AsPath {
            for a in self.get_as_paths_mut() {
                if a.get_as_path() == as_path {
                    return a;
                }
            }
            panic!("AS Path not found {:?}", as_path);
        }

        fn get_origin(&self) -> &Asn {
            &self.origin
        }

        // pub fn get_routes_at_path(&self, path: &Vec<Asn>) -> &AsPathRoutes {
        //     self.paths.get(path).unwrap()
        // }

        pub fn has_as_path(&self, as_path: &Vec<Asn>) -> bool {
            let mut present = false;
            for a in self.get_as_paths() {
                if a.get_as_path() == as_path {
                    present = true;
                    break;
                }
            }
            debug!("AS path present {:?}: {}", as_path, present);
            present
        }

        pub fn has_route(&self, route: &Route) -> bool {
            if route.get_origin() != self.get_origin() { panic!("Checking if route exists in AS Paths for origin {}: {:?}", self.get_origin(), route) };
            if !self.has_as_path(route.get_as_path()) { return false; };
            let as_path = self.get_as_path(route.get_as_path());
            as_path.has_route(route)
        }

        // pub fn insert_as_path(&mut self, as_path: Vec<Asn>) {
        //     debug!("Adding AS path: {:?}", &as_path);
        //     let asp = AsPath::new(as_path);
        //     if asp.get_origin() != self.get_origin() {
        //         panic!("Can't add AS path with incorrect origin, expected {} in {:?}", self.get_origin(), asp.get_as_path());
        //     }
        //     if !self.has_as_path(&asp) {
        //         self.as_paths.push(asp);
        //     }
        // }

        // pub fn insert_route_at_path(&mut self, path: Vec<Asn>, route: Route) {
        //     debug!("Adding route in AS path {:?}: {:?}", path, route);
        //     if !self.has_path(&path) {
        //         self.insert_path(path.clone());
        //     }

        //     self.paths.get_mut(&path).unwrap().insert_route(route);
        // }
    }

}
