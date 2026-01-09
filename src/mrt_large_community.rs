pub mod large_community {
    use bgpkit_parser::models::LargeCommunity;

    pub trait Testing {
        fn get_mock(large_community: Option<(u32, u32, u32)>) -> LargeCommunity;
    }

    impl Testing for LargeCommunity {
        fn get_mock(large_community: Option<(u32, u32, u32)>) -> LargeCommunity {
            let parts = large_community.unwrap_or((65535, 65535, 23456));
            LargeCommunity::new(parts.0, [parts.1, parts.2])
        }
    }
}

pub mod large_communities {
    use bgpkit_parser::models::LargeCommunity;

    pub fn get_mock(large_community: Option<(u32, u32, u32)>) -> Vec<LargeCommunity> {
        let parts = large_community.unwrap_or((65535, 65535, 23456));

        Vec::from([
            LargeCommunity::new(1, [1, 1]),
            LargeCommunity::new(2, [2, 2]),
            LargeCommunity::new(parts.0, [parts.1, parts.2]),
        ])
    }
}
