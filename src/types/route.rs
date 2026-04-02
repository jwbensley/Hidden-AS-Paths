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
    origin: Asn,
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
        let mut state = serializer.serialize_struct("Route", 1)?;
        // state.serialize_field("origin", &self.origin)?;
        // state.serialize_field("filename", &self.filename)?;
        // state.serialize_field("next_hop", &self.next_hop)?;
        // state.serialize_field("peer", &self.peer)?;
        // state.serialize_field("prefix", &self.prefix.to_string())?;
        state.serialize_field("communities", &self.communities)?;
        state.end()
    }
}

impl Route {
    pub fn new(
        origin: Asn,
        filename: String,
        next_hop: IpAddr,
        peer: Peer,
        prefix: IpNet,
        communities: Vec<StandardCommunity>,
    ) -> Self {
        Self {
            origin,
            filename,
            next_hop,
            peer,
            prefix,
            communities,
        }
    }

    pub fn get_mock(origin: Option<Asn>) -> Self {
        let origin = origin.unwrap_or(Asn::get_mock(None));

        Self {
            origin,
            filename: String::from("mock_filename"),
            next_hop: "127.0.0.1".parse().unwrap(),
            peer: Peer::get_mock(),
            prefix: "127.0.0.0/8".parse().unwrap(),
            communities: Vec::from([StandardCommunity::get_mock(None)]),
        }
    }

    fn get_communities(&self) -> &Vec<StandardCommunity> {
        &self.communities
    }

    pub fn get_origin(&self) -> &Asn {
        &self.origin
    }

    pub fn get_prefix(&self) -> &IpNet {
        &self.prefix
    }

    pub fn has_unknown_community_asns(&self, known_asns: &[Asn]) -> bool {
        // Communities with a private ASN are not necessarily "unknown", they may just be internally used communities
        // which have leaked into the public internet. Therefore, don't consider private ASNs as unknown.
        for community in self.get_communities() {
            if !known_asns.contains(community.get_asn()) && !community.get_asn().is_private() {
                return true;
            }
        }
        false
    }

    pub fn remove_communities_with_known_asns(&mut self, known_asns: &[Asn]) {
        self.communities
            .retain(|community| !known_asns.contains(community.get_asn()));
    }
}

#[cfg(test)]
mod tests {
    use crate::types::as_path::AsPath;

    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_new() {
        let origin = Asn::get_mock(None);
        let filename = String::from("test_file.mrt");
        let next_hop: IpAddr = "192.0.2.1".parse().unwrap();
        let peer = Peer::get_mock();
        let prefix: IpNet = "10.0.0.0/8".parse().unwrap();
        let communities = vec![StandardCommunity::get_mock(None)];

        let route = Route::new(
            origin.clone(),
            filename.clone(),
            next_hop,
            peer.clone(),
            prefix,
            communities.clone(),
        );

        assert_eq!(route.origin, origin);
        assert_eq!(route.filename, filename);
        assert_eq!(route.next_hop, next_hop);
        assert_eq!(route.peer, peer);
        assert_eq!(route.prefix, prefix);
        assert_eq!(route.communities, communities);
    }

    #[test]
    fn test_get_mock_with_none() {
        let route = Route::get_mock(None);
        assert_eq!(route.origin, Asn::get_mock(None));
        assert_eq!(route.filename, "mock_filename");
        assert_eq!(route.next_hop, "127.0.0.1".parse::<IpAddr>().unwrap());
        assert_eq!(route.peer, Peer::get_mock());
        assert_eq!(route.prefix, "127.0.0.0/8".parse::<IpNet>().unwrap());
        assert_eq!(
            route.communities,
            Vec::from([StandardCommunity::get_mock(None)])
        );
    }

    #[test]
    fn test_get_mock_with_some() {
        let origin = Asn::new(123);
        let route = Route::get_mock(Some(origin.clone()));
        assert_eq!(route.get_origin(), &origin);
    }

    #[test]
    fn test_get_origin() {
        let route = Route::get_mock(None);
        assert_eq!(
            route.get_origin(),
            AsPath::get_mock(None, None).get_origin()
        );
    }

    #[test]
    fn test_get_prefix() {
        let prefix: IpNet = "192.168.0.0/16".parse().unwrap();
        let route = Route::new(
            Asn::get_mock(None),
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
        let origin = Asn::get_mock(None);
        let communities = Vec::from([StandardCommunity::get_mock(None)]);
        let peer = Peer::get_mock();

        let route = Route::new(
            origin.clone(),
            String::from("test.mrt"),
            "192.0.2.1".parse().unwrap(),
            peer.clone(),
            "10.0.0.0/8".parse().unwrap(),
            communities.clone(),
        );

        let json = serde_json::to_string(&route).unwrap();
        let expected_json = "{\"origin\":".to_owned()
            + &serde_json::to_string(&origin).unwrap()
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
            Asn::new(55),
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
            Asn::new(55),
            String::from("different.mrt"),
            "192.0.2.2".parse().unwrap(),
            Peer::get_mock(),
            "10.0.0.0/8".parse().unwrap(),
            Vec::new(),
        );

        assert!(!set.contains(&route3));
    }
}
