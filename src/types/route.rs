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
        self.as_path.get_asns().last().unwrap()
    }

    pub fn get_prefix(&self) -> &IpNet {
        &self.prefix
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_new() {
        let as_path = AsPath::get_mock(None);
        let filename = String::from("test_file.mrt");
        let next_hop: IpAddr = "192.0.2.1".parse().unwrap();
        let peer = Peer::get_mock();
        let prefix: IpNet = "10.0.0.0/8".parse().unwrap();
        let communities = vec![StandardCommunity::get_mock(None)];

        let route = Route::new(
            as_path.clone(),
            filename.clone(),
            next_hop,
            peer.clone(),
            prefix,
            communities.clone(),
        );

        assert_eq!(route.as_path, as_path);
        assert_eq!(route.filename, filename);
        assert_eq!(route.next_hop, next_hop);
        assert_eq!(route.peer, peer);
        assert_eq!(route.prefix, prefix);
        assert_eq!(route.communities, communities);
    }

    #[test]
    fn test_get_mock_with_none() {
        let route = Route::get_mock(None);
        assert_eq!(route.filename, "mock_filename");
        assert_eq!(route.next_hop, "127.0.0.1".parse::<IpAddr>().unwrap());
        assert_eq!(route.peer, Peer::get_mock());
        assert_eq!(route.prefix, "127.0.0.0/8".parse::<IpNet>().unwrap());
        assert_eq!(route.communities.len(), 0);
    }

    #[test]
    fn test_get_mock_with_some() {
        let custom_as_path = AsPath::get_mock(Some(vec![Asn::new(100), Asn::new(200)]));
        let route = Route::get_mock(Some(custom_as_path.clone()));
        assert_eq!(route.as_path, custom_as_path);
    }

    #[test]
    fn test_get_as_path() {
        let as_path = AsPath::get_mock(None);
        let route = Route::get_mock(Some(as_path.clone()));
        assert_eq!(route.get_as_path(), &as_path);
    }

    #[test]
    fn test_get_origin() {
        let as_path = AsPath::get_mock(None);
        let route = Route::get_mock(Some(as_path.clone()));
        assert_eq!(route.get_origin(), as_path.get_origin());
    }

    #[test]
    fn test_get_prefix() {
        let prefix: IpNet = "192.168.0.0/16".parse().unwrap();
        let route = Route::new(
            AsPath::get_mock(None),
            String::from("test.mrt"),
            "192.0.2.1".parse().unwrap(),
            Peer::get_mock(),
            prefix,
            Vec::new(),
        );

        assert_eq!(route.get_prefix(), &prefix);
    }

    #[test]
    fn test_serialize() {
        let as_path = AsPath::get_mock(None);
        let communities = Vec::from([StandardCommunity::get_mock(None)]);
        let peer = Peer::get_mock();

        let route = Route::new(
            as_path.clone(),
            String::from("test.mrt"),
            "192.0.2.1".parse().unwrap(),
            peer.clone(),
            "10.0.0.0/8".parse().unwrap(),
            communities.clone(),
        );

        let json = serde_json::to_string(&route).unwrap();
        let expected_json = "{\"as_path\":".to_owned()
            + &serde_json::to_string(&as_path).unwrap()
            + ",\"filename\":\"test.mrt\",\"next_hop\":\"192.0.2.1\",\"peer\":"
            + &serde_json::to_string(&peer).unwrap()
            + ",\"prefix\":\"10.0.0.0/8\",\"communities\":"
            + &serde_json::to_string(&communities).unwrap()
            + "}";
        assert!(json == expected_json);
    }

    #[test]
    fn test_clone() {
        let route1 = Route::get_mock(None);
        let route2 = route1.clone();
        assert_eq!(route1, route2);
    }

    #[test]
    fn test_partial_eq() {
        let route1 = Route::get_mock(None);
        let route2 = Route::get_mock(None);

        assert_eq!(route1, route2);

        let route3 = Route::new(
            AsPath::get_mock(Some(vec![Asn::new(999)])),
            String::from("different.mrt"),
            "192.0.2.2".parse().unwrap(),
            Peer::get_mock(),
            "10.0.0.0/8".parse().unwrap(),
            Vec::new(),
        );

        assert_ne!(route1, route3);
    }

    #[test]
    fn test_hash() {
        let route1 = Route::get_mock(None);
        let route2 = Route::get_mock(None);

        let mut set = HashSet::new();
        set.insert(route1.clone());

        assert!(set.contains(&route2));

        let route3 = Route::new(
            AsPath::get_mock(Some(vec![Asn::new(999)])),
            String::from("different.mrt"),
            "192.0.2.2".parse().unwrap(),
            Peer::get_mock(),
            "10.0.0.0/8".parse().unwrap(),
            Vec::new(),
        );

        assert!(!set.contains(&route3));
    }
}
