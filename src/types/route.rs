use crate::types::as_path::AsPath;
use crate::types::asn::Asn;
use crate::types::community::StandardCommunity;
use crate::types::peer::Peer;
use ipnet::IpNet;
use serde::ser::SerializeStruct as _;
use serde::{Serialize, Serializer};
use std::hash::Hash;
use std::net::IpAddr;

/// Store a route pulled from an MRT file and provide serialisation to JSON.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Route {
    as_path: AsPath,
    filename: String,
    next_hop: IpAddr,
    peer: Peer,
    prefix: IpNet,
    communities: Vec<StandardCommunity>,
}

impl Serialize for Route {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Route", 11)?;
        state.serialize_field("as_path", &self.as_path)?;
        state.serialize_field("filename", &self.filename)?;
        state.serialize_field("next_hop", &self.next_hop)?;
        state.serialize_field("peer", &self.peer)?;
        state.serialize_field("prefix", &self.prefix.to_string())?;
        state.serialize_field("communities", &self.communities)?;
        state.end()
    }
}

impl Route {
    pub fn new(
        as_path: AsPath,
        filename: String,
        next_hop: IpAddr,
        peer: Peer,
        prefix: IpNet,
        communities: Vec<StandardCommunity>,
    ) -> Self {
        Self {
            as_path,
            filename,
            next_hop,
            peer,
            prefix,
            communities,
        }
    }

    pub fn get_mock(origin: Option<AsPath>) -> Self {
        let as_path = origin.unwrap_or_else(|| AsPath::get_mock(None));

        Self {
            as_path,
            filename: String::from("mock_filename"),
            next_hop: "127.0.0.1".parse().unwrap(),
            peer: Peer::get_mock(),
            prefix: "127.0.0.0/8".parse().unwrap(),
            communities: Vec::new(),
        }
    }

    pub fn get_as_path(&self) -> &AsPath {
        &self.as_path
    }

    pub fn get_origin(&self) -> &Asn {
        self.as_path.get_as_path().last().unwrap()
    }

    pub fn get_prefix(&self) -> &IpNet {
        &self.prefix
    }
}
