use bgpkit_parser::models::Asn as BgpkitAsn;
use serde::{Serialize, Serializer};
use std::fmt;

/// Aa wrapper around the `Asn` type from `bgpkit_parser` to allow for serialisation to JSON.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Asn(BgpkitAsn);

impl fmt::Display for Asn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for Asn {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u32(self.0.to_u32())
    }
}

impl Asn {
    pub fn new(asn: u32) -> Self {
        Self(BgpkitAsn::new_32bit(asn))
    }

    pub fn get_mock(asn: Option<u32>) -> Asn {
        Asn::new(asn.unwrap_or(65535))
    }

    pub fn to_u32(self) -> u32 {
        self.0.to_u32()
    }
}
