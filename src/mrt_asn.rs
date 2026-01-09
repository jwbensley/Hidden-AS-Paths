pub mod asn {
    use bgpkit_parser::models::Asn;

    pub trait Testing {
        fn get_mock(asn: Option<u32>) -> Asn;
    }

    impl Testing for Asn {
        fn get_mock(asn: Option<u32>) -> Asn {
            Asn::new_32bit(asn.unwrap_or(65535))
        }
    }
}
