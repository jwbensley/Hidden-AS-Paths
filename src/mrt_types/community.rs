use crate::mrt_types::asn::Asn;
use bgpkit_parser::models::Community;
use serde::ser::SerializeTuple;
use serde::{Serialize, Serializer};
use std::hash::Hash;

/// Standard community which can be serialised to JSON.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StandardCommunity {
    asn: Asn,
    value: u16,
}

impl Serialize for StandardCommunity {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut tup = serializer.serialize_tuple(2)?;
        tup.serialize_element(&self.get_asn().clone().to_u32())?;
        tup.serialize_element(&self.get_value())?;
        tup.end()
    }
}

impl Hash for StandardCommunity {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.asn.clone().to_u32().hash(state);
        self.value.hash(state);
    }
}

impl StandardCommunity {
    pub fn new(asn: Asn, value: u16) -> Self {
        Self { asn, value }
    }

    pub fn from_community(community: &Community) -> Self {
        if let Community::Custom(asn, value) = community {
            StandardCommunity::new(Asn::new(asn.to_u32()), *value)
        } else if let Community::NoAdvertise = community {
            StandardCommunity::new(Asn::new(65535), 65281)
        } else if let Community::NoExport = community {
            StandardCommunity::new(Asn::new(65535), 65282)
        } else if let Community::NoExportSubConfed = community {
            StandardCommunity::new(Asn::new(65535), 65283)
        } else {
            panic!(
                "Couldn't unpack Community into StandardCommunity: {}",
                community
            );
        }
    }

    pub fn get_mock(community: Option<(Asn, u16)>) -> StandardCommunity {
        let parts = community.unwrap_or((Asn::get_mock(None), 23456));
        StandardCommunity::new(parts.0, parts.1)
    }

    pub fn get_asn(&self) -> &Asn {
        &self.asn
    }

    fn get_value(&self) -> &u16 {
        &self.value
    }
}
