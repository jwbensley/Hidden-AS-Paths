pub mod community {
    use bgpkit_parser::models::{Asn, Community};

    pub trait Testing {
        fn get_mock(community: Option<(Asn, u16)>) -> Community;
    }

    impl Testing for Community {
        fn get_mock(community: Option<(Asn, u16)>) -> Community {
            let parts = community.unwrap_or((Asn::new_32bit(65535), 23456));
            Community::Custom(parts.0, parts.1)
        }
    }
}

pub mod communities {
    use std::vec::Vec;

    use bgpkit_parser::models::{Asn, Community};

    pub fn get_mock(community: Option<(Asn, u16)>) -> Vec<Community> {
        let parts = community.unwrap_or((Asn::new_32bit(65535), 23456));

        Vec::from([
            Community::Custom(Asn::new_32bit(1), 1),
            Community::Custom(Asn::new_32bit(2), 2),
            Community::Custom(parts.0, parts.1),
        ])
    }
}
