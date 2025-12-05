pub mod asp_trees {
    use bgpkit_parser::models::Asn;
    use bgpkit_parser::models::Peer;
    use ipnet::IpNet;
    use log::{debug, info};
    use std::collections::HashMap;
    use std::collections::hash_map::Keys;
    use std::net::IpAddr;

    #[derive(Clone, Debug)]
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

    /*
     * A list of routes
     */
    #[derive(Debug)]
    pub struct AsPathRoutes {
        routes: Vec<Route>,
    }

    impl Default for AsPathRoutes {
        fn default() -> Self {
            Self::new()
        }
    }

    impl PartialEq for AsPathRoutes {
        fn eq(&self, other: &Self) -> bool {
            self.routes == other.routes
        }
    }

    impl AsPathRoutes {
        pub fn new() -> Self {
            AsPathRoutes {
                routes: Vec::<Route>::new(),
            }
        }

        pub fn len(&self) -> usize {
            self.routes.len()
        }

        fn get_routes(&self) -> &Vec<Route> {
            &self.routes
        }

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

    /*
     * A set of AS paths, each contains a set of routes
     */
    #[derive(Debug)]
    pub struct AsPaths {
        paths: HashMap<Vec<Asn>, AsPathRoutes>,
    }

    impl Default for AsPaths {
        fn default() -> Self {
            Self::new()
        }
    }

    impl PartialEq for AsPaths {
        fn eq(&self, other: &Self) -> bool {
            self.paths == other.paths
        }
    }

    impl AsPaths {
        pub fn is_empty(&self) -> bool {
            self.paths.len() == 0
        }

        pub fn len(&self) -> usize {
            self.paths.len()
        }

        pub fn new() -> Self {
            AsPaths {
                paths: HashMap::<Vec<Asn>, AsPathRoutes>::new(),
            }
        }

        fn get_paths(&self) -> Keys<'_, Vec<Asn>, AsPathRoutes> {
            self.paths.keys()
        }

        fn get_routes_at_path(&self, path: &Vec<Asn>) -> &AsPathRoutes {
            self.paths.get(path).unwrap()
        }

        fn has_path(&self, path: &Vec<Asn>) -> bool {
            let present = self.paths.contains_key(path);
            debug!("AS path present {:?}: {}", path, present);
            present
        }

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

            self.paths.get_mut(&path).unwrap().insert_route(route);
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

    impl PartialEq for AsSequences {
        fn eq(&self, other: &Self) -> bool {
            self.sequences == other.sequences
        }
    }

    impl AsSequences {
        pub fn new() -> Self {
            AsSequences {
                sequences: HashMap::<Vec<Asn>, AsPaths>::new(),
            }
        }

        fn get_as_paths_at_sequence(&self, sequence: &Vec<Asn>) -> &AsPaths {
            self.sequences.get(sequence).unwrap()
        }

        pub fn get_sequences(&self) -> Keys<'_, Vec<Asn>, AsPaths> {
            self.sequences.keys()
        }

        fn has_sequence(&self, sequence: &Vec<Asn>) -> bool {
            debug!(
                "AS sequence present {:?}: {}",
                sequence,
                self.sequences.contains_key(sequence),
            );
            self.sequences.contains_key(sequence)
        }

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

            self.sequences
                .get_mut(&sequence)
                .unwrap()
                .insert_route_at_path(path, route);
        }

        pub fn merge_from(&mut self, other: &Self) {
            for other_sequence in other.get_sequences() {
                let other_as_paths = other.get_as_paths_at_sequence(other_sequence);

                for other_as_path in other_as_paths.get_paths() {
                    let other_routes = other_as_paths.get_routes_at_path(&other_as_path);

                    for route in other_routes.get_routes() {
                        self.insert_route_at_sequence(
                            other_sequence.clone(),
                            other_as_path.clone(),
                            route.clone(),
                        );
                    }
                }
            }
        }

        pub fn print_as_paths(&self) {
            for sequence in self.get_sequences() {
                info!("{:?}", sequence);
                for path in self.get_as_paths_at_sequence(sequence).get_paths() {
                    info!("    {:?}", path);
                }
            }
        }

        pub fn print_total(&self) {
            let mut sequences = 0;
            let mut paths = 0;
            let mut routes = 0;

            for sequence in self.get_sequences() {
                sequences += 1;
                let as_paths = self.get_as_paths_at_sequence(sequence);
                for path in as_paths.get_paths() {
                    paths += 1;
                    routes += as_paths.get_routes_at_path(path).len();
                }
            }
            info!(
                "{} deduped paths, {} paths, {} routes",
                sequences, paths, routes
            );
        }

        pub fn remove_single_paths(&mut self) {
            let mut to_remove = Vec::new();
            for sequence in self.get_sequences() {
                if self.get_as_paths_at_sequence(sequence).len() == 1 {
                    to_remove.push(sequence.to_owned());
                }
            }
            for key in to_remove.iter() {
                self.sequences.remove(key);
            }
        }
    }

    pub fn merge_sequences(mut as_sequences: Vec<AsSequences>) -> AsSequences {
        /*
         * Merge pairs of AsSequences, delete the 2nd item from each pair,
         * merge the remaing items in pairs...Continue until only one item is left.
         */
        info!("Merging {} sequences", as_sequences.len());

        if as_sequences.is_empty() {
            panic!("No sequences to merge!");
        } else if as_sequences.len() == 1 {
            debug!("Only 1 item, nothing to merge");
            return as_sequences.pop().unwrap();
        }

        while as_sequences.len() > 1 {
            for chunks in as_sequences.chunks_mut(2) {
                if let [seq1, seq2] = chunks {
                    seq1.merge_from(seq2);
                }
            }

            let to_delete = as_sequences.len() / 2; // rounds down
            let mut deleted = 0;
            let mut index = 1;

            while deleted < to_delete {
                for i in 0..as_sequences.len() {
                    // ^ Up to but not including

                    if i == index {
                        as_sequences.remove(i);
                        deleted += 1;
                        index += 1;
                        break;
                    }
                }
            }
        }
        as_sequences.pop().unwrap()
    }
}
