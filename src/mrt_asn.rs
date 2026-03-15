/// This modules provides a wrapper around the `Asn` type from `bgpkit_parser` to allow for serialisation to JSON.
pub mod asn {
    use std::fmt;

    use bgpkit_parser::models::Asn;
    use serde::{Serialize, Serializer};

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct MrtAsn(Asn);

    impl fmt::Display for MrtAsn {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl Serialize for MrtAsn {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_u32(self.0.to_u32())
        }
    }

    impl MrtAsn {
        pub fn new(asn: u32) -> Self {
            Self(Asn::new_32bit(asn))
        }

        pub fn get_mock(asn: Option<u32>) -> MrtAsn {
            MrtAsn::new(asn.unwrap_or(65535))
        }

        pub fn to_u32(self) -> u32 {
            self.0.to_u32()
        }
    }
}
