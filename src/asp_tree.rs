pub mod asp_trees {
    use bgpkit_parser::models::Asn;
    use bgpkit_parser::models::Peer;
    use ipnet::IpNet;
    use log::debug;
    use std::collections::HashMap;
    use std::net::IpAddr;

    #[derive(Debug)]
    pub struct Route {
        aspath: Vec<Asn>,
        aspath_deduped: Vec<Asn>,
        filename: String,
        next_hop: IpAddr,
        peer: Peer,
        prefix: IpNet,
    }

    impl Route {
        pub fn new(
            aspath: Vec<Asn>,
            aspath_deduped: Vec<Asn>,
            filename: String,
            next_hop: IpAddr,
            peer: Peer,
            prefix: IpNet,
        ) -> Self {
            Self {
                aspath,
                aspath_deduped,
                filename,
                next_hop,
                peer,
                prefix,
            }
        }
    }

    impl PartialEq for Route {
        fn eq(&self, other: &Self) -> bool {
            (self.aspath == other.aspath)
                && (self.aspath_deduped == other.aspath_deduped)
                && (self.filename == other.filename)
                && (self.next_hop == other.next_hop)
                && (self.peer == other.peer)
                && (self.prefix == other.prefix)
        }
    }

    #[derive(Debug)]
    pub struct Paths {
        paths: HashMap<Vec<Asn>, HashMap<Vec<Asn>, Vec<Route>>>,
    }

    impl Default for Paths {
        fn default() -> Self {
            Self::new()
        }
    }

    impl Paths {
        pub fn new() -> Self {
            Paths {
                paths: HashMap::<Vec<Asn>, HashMap<Vec<Asn>, Vec<Route>>>::new(),
            }
        }

        fn has_deduped_path(&self, deduped_path: &Vec<Asn>) -> bool {
            debug!(
                "Deduped AS sequence present: {}, {:?}",
                self.paths.contains_key(deduped_path),
                deduped_path
            );
            self.paths.contains_key(deduped_path)
        }

        fn insert_deduped_path(&mut self, deduped_path: Vec<Asn>) {
            debug!("Adding deduped AS sequence: {:?}", deduped_path);
            self.paths.insert(deduped_path, HashMap::new());
        }

        fn has_path(&self, deduped_path: &Vec<Asn>, path: &Vec<Asn>) -> bool {
            debug!(
                "AS sequence present in {:?}: {}, {:?}",
                deduped_path,
                self.paths[deduped_path].contains_key(path),
                path
            );
            self.paths[deduped_path].contains_key(path)
        }

        fn insert_path(&mut self, deduped_path: &Vec<Asn>, path: Vec<Asn>) {
            debug!("Adding AS sequence in {:?}: {:?}", deduped_path, path);
            self.paths
                .get_mut(deduped_path)
                .unwrap()
                .insert(path, Vec::new());
        }

        fn has_route(&self, deduped_path: &Vec<Asn>, path: &Vec<Asn>, route: &Route) -> bool {
            debug!(
                "Route sequence present in {:?}, {:?}: {}, {:?}",
                deduped_path,
                path,
                self.paths[deduped_path][path].contains(route),
                route
            );
            self.paths[deduped_path][path].contains(route)
        }

        fn insert_route(&mut self, deduped_path: &Vec<Asn>, path: &Vec<Asn>, route: Route) {
            debug!(
                "Adding route in {:?}, {:?}: {:?}",
                deduped_path, path, route
            );
            self.paths
                .get_mut(deduped_path)
                .unwrap()
                .get_mut(path)
                .unwrap()
                .push(route);
        }

        pub fn insert_route_from_root(
            &mut self,
            deduped_path: Vec<Asn>,
            path: Vec<Asn>,
            route: Route,
        ) {
            if !self.has_deduped_path(&deduped_path) {
                self.insert_deduped_path(deduped_path.clone());
            }
            if !self.has_path(&deduped_path, &path) {
                self.insert_path(&deduped_path, path.clone());
            }
            if !self.has_route(&deduped_path, &path, &route) {
                self.insert_route(&deduped_path, &path, route);
            }
        }
    }
}
