pub mod ip_net {
    use std::net::IpAddr;

    use ipnet::IpNet;

    pub trait Testing {
        fn get_mock() -> IpNet;
    }

    impl Testing for IpNet {
        fn get_mock() -> IpNet {
            IpNet::new(IpAddr::from([192, 0, 2, 1]), 24).unwrap()
        }
    }
}
