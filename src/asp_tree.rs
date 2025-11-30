pub mod asp_trees {
    use bgpkit_parser::models::Asn;
    use bgpkit_parser::models::Peer;
    use ipnet::IpNet;
    use log::debug;
    use std::collections::HashMap;
    use std::collections::hash_map::Keys;
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
    pub struct AsPathRoutes {
        routes: Vec<Route>,
    }

    impl Default for AsPathRoutes {
        fn default() -> Self {
            Self::new()
        }
    }

    impl AsPathRoutes {
        pub fn new() -> Self {
            AsPathRoutes {
                routes: Vec::<Route>::new(),
            }
        }

        // fn get_routes(&self) -> &Vec<Route> {
        //     &self.routes
        // }

        fn has_route(&self, route: &Route) -> bool {
            let present = self.routes.contains(route);
            debug!("Route present {:?}: {}", route, present);
            present
        }

        fn insert_route(&mut self, route: Route) {
            debug!("Adding route: {:?}", route);
            if !self.has_route(&route) {
                self.routes.push(route);
            }
        }
    }

    #[derive(Debug)]
    pub struct AsPaths {
        paths: HashMap<Vec<Asn>, AsPathRoutes>,
    }

    impl Default for AsPaths {
        fn default() -> Self {
            Self::new()
        }
    }

    impl AsPaths {
        pub fn new() -> Self {
            AsPaths {
                paths: HashMap::<Vec<Asn>, AsPathRoutes>::new(),
            }
        }

        fn get_routes_at_path(&mut self, path: &Vec<Asn>) -> &mut AsPathRoutes {
            self.paths.get_mut(path).unwrap()
        }

        fn has_path(&self, path: &Vec<Asn>) -> bool {
            let present = self.paths.contains_key(path);
            debug!("AS path present {:?}: {}", path, present);
            present
        }

        // fn has_route(&mut self, path: &Vec<Asn>, route: &Route) -> bool {
        //     let mut present = false;

        //     if self.has_path(path) {
        //         present = self.get_routes_at_path(path).has_route(route);
        //     }

        //     debug!(
        //         "Route present in AS path {:?}: {}, {:?}",
        //         path, present, route,
        //     );

        //     present
        // }

        fn insert_path(&mut self, path: Vec<Asn>) {
            debug!("Adding AS path: {:?}", path);
            if !self.has_path(&path) {
                self.paths.insert(path, AsPathRoutes::new());
            }
        }

        fn insert_route_at_path(&mut self, path: Vec<Asn>, route: Route) {
            debug!("Adding route in AS path {:?}: {:?}", path, route);
            if !self.has_path(&path) {
                self.insert_path(path.clone());
            }
            self.get_routes_at_path(&path).insert_route(route);
        }
    }

    /*
     * A set of deduped AS numbers, each contains a set of non-deduped AS paths
     */
    #[derive(Debug)]
    pub struct AsSequences {
        sequences: HashMap<Vec<Asn>, AsPaths>,
    }

    impl Default for AsSequences {
        fn default() -> Self {
            Self::new()
        }
    }

    impl AsSequences {
        pub fn new() -> Self {
            AsSequences {
                sequences: HashMap::<Vec<Asn>, AsPaths>::new(),
            }
        }

        fn get_as_paths_at_sequence(&mut self, sequence: &Vec<Asn>) -> &mut AsPaths {
            self.sequences.get_mut(sequence).unwrap()
        }

        fn get_sequences(&self) -> Vec<Asn> {
            self.sequences.keys().cloned().collect()
        }

        // fn has_as_paths_at_sequence(&self, sequence: &Vec<Asn>, path: &Vec<Asn>) -> bool {
        //     let mut present = false;

        //     if self.has_sequence(sequence) {
        //         present = self.sequences[sequence].has_path(path);
        //     }

        //     debug!(
        //         "AS sequence has AS path {:?}: {}, {:?}",
        //         sequence, present, path
        //     );
        //     present
        // }

        fn has_sequence(&self, sequence: &Vec<Asn>) -> bool {
            debug!(
                "AS sequence present {:?}: {}",
                sequence,
                self.sequences.contains_key(sequence),
            );
            self.sequences.contains_key(sequence)
        }

        // fn insert_as_paths_at_sequence(&mut self, sequence: Vec<Asn>, path: Vec<Asn>) {
        //     debug!("Adding AS path for AS sequence {:?}: {:?}", sequence, path);
        //     if !self.has_sequence(&sequence) {
        //         self.insert_sequence(sequence.clone());
        //     }
        //     self.get_as_paths_at_sequence(&sequence).insert_path(path);
        // }

        fn insert_sequence(&mut self, sequence: Vec<Asn>) {
            debug!("Adding AS sequence: {:?}", sequence);
            if !self.has_sequence(&sequence) {
                self.sequences.insert(sequence, AsPaths::new());
            }
        }

        pub fn insert_route_at_sequence(
            &mut self,
            sequence: Vec<Asn>,
            path: Vec<Asn>,
            route: Route,
        ) {
            debug!(
                "Adding route, with AS Path, under AS sequence: {:?}, {:?}, {:?}",
                sequence, path, route
            );
            if !self.has_sequence(&sequence) {
                self.insert_sequence(sequence.clone());
            }

            self.get_as_paths_at_sequence(&sequence)
                .insert_route_at_path(path, route);
        }

        pub fn merge_from(&mut self, other: &Self) {
            for sequence in other.get_sequences() {
                //self.insert_sequence(sequence.clone());
            }
        }
    }
}
