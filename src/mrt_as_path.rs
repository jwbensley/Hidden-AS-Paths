pub mod as_path {
    use crate::mrt_route::route::Route;
    use bgpkit_parser::models::Asn;
    use log::debug;

    /// A deduped AS path which stores one or more routes
    #[derive(Debug, Clone)]
    pub struct AsPath {
        as_path: Vec<Asn>,
        routes: Vec<Route>,
    }

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

        pub fn get_as_path(&self) -> &Vec<Asn> {
            &self.as_path
        }

        pub fn has_route(&self, route: &Route) -> bool {
            let present = self.routes.contains(route);
            debug!("Route present {:#?}: {}", route, present);
            present
        }
    }
}
