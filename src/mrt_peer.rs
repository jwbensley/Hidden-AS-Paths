pub mod peer {
    use bgpkit_parser::models::{Asn, Peer};
    use std::net::{IpAddr, Ipv4Addr};

    pub trait Testing {
        fn get_mock() -> Peer;
    }

    impl Testing for Peer {
        fn get_mock() -> Peer {
            Peer::new(
                Ipv4Addr::new(192, 0, 2, 1),
                IpAddr::from([192, 0, 2, 1]),
                Asn::new_32bit(65535),
            )
        }
    }
}
