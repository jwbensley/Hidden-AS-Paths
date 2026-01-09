pub mod ip_addr {
    use std::net::IpAddr;

    pub trait Testing {
        fn get_mock() -> IpAddr;
    }

    impl Testing for IpAddr {
        fn get_mock() -> IpAddr {
            IpAddr::from([192, 0, 2, 1])
        }
    }
}
