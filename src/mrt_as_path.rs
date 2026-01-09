pub mod as_path {
    use crate::mrt_asn::asn::Testing;
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

        pub fn get_mock(origin: Option<Asn>) -> AsPath {
            let as_path = Vec::from([
                Asn::get_mock(Some(1)),
                Asn::get_mock(Some(2)),
                origin.unwrap_or(Asn::get_mock(None)),
            ]);
            AsPath::new(Vec::from(as_path))
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

        fn get_asns(&self) -> &Vec<Asn> {
            &self.as_path
        }

        /// Overlap means that both AS paths have an ASN which occurs somewhere in both paths,
        /// but not the last ASN (the origin), that is gaurenteed to be the same ASN.
        pub fn has_overlap_with(&self, other: &AsPath) -> bool {
            assert_ne!(self, other, "Trying to compare the same AS Paths");

            for asn in self.get_asns().split_last().unwrap().1 {
                if other.get_asns().split_last().unwrap().1.contains(asn) {
                    return true;
                }
            }
            false
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
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_as_path_eq() {
            let mut ap_1 = AsPath::get_mock(None);
            ap_1.add_route(Route::get_mock(None));

            let mut ap_2 = AsPath::get_mock(None);
            ap_2.add_route(Route::get_mock(None));

            assert_eq!(ap_1, ap_2);

            ap_1 = AsPath::get_mock(Some(Asn::new_32bit(1)));
            ap_1.add_route(Route::get_mock(Some(Asn::new_32bit(1))));

            ap_2 = AsPath::get_mock(Some(Asn::new_32bit(1)));
            ap_2.add_route(Route::get_mock(Some(Asn::new_32bit(1))));

            assert_eq!(ap_1, ap_2);
        }

        #[test]
        fn test_as_path_ne() {
            // NE with different origins
            let mut ap_1 = AsPath::get_mock(Some(Asn::new_32bit(1)));
            let mut ap_2 = AsPath::get_mock(Some(Asn::new_32bit(2)));
            assert_ne!(ap_1, ap_2);

            // NE if missing Route
            ap_1 = AsPath::get_mock(Some(Asn::new_32bit(1)));
            ap_1.add_route(Route::get_mock(Some(Asn::new_32bit(1))));
            ap_2 = AsPath::get_mock(Some(Asn::new_32bit(1)));
            assert_ne!(ap_1, ap_2);

            // NE if different routes
            ap_1 = AsPath::get_mock(Some(Asn::new_32bit(1)));
            ap_1.add_route(Route::get_mock(Some(Asn::new_32bit(1))));
            ap_2 = AsPath::get_mock(Some(Asn::new_32bit(1)));
            ap_1.add_route(Route::get_mock(Some(Asn::new_32bit(2))));
            assert_ne!(ap_1, ap_2);
        }
    }
}
