use bgpkit_parser::models::{Asn as BgpKitAsn, Peer as BgpKitPeer};
use serde::{Serialize, Serializer, ser::SerializeStruct};
use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr},
};

/// The Peer and PeerTable, which are used to represent BGP peers in MRT files.
/// The Peer struct wraps a BgpKit_Peer to allow for serialisation to JSON.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Peer(BgpKitPeer);

impl Serialize for Peer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Peer", 4)?;
        state.serialize_field("peer_bgp_id", &self.0.peer_bgp_id)?;
        state.serialize_field("peer_ip", &self.0.peer_ip)?;
        state.serialize_field("peer_asn", &self.0.peer_asn.to_u32())?;
        state.end()
    }
}

impl Peer {
    pub fn new(peer: BgpKitPeer) -> Self {
        Self(peer)
    }

    pub fn get_mock() -> Peer {
        let mut pt: bgpkit_parser::models::PeerType = bgpkit_parser::models::PeerType::empty();
        pt.insert(bgpkit_parser::models::PeerType::AS_SIZE_32BIT);
        Peer::new(BgpKitPeer {
            peer_type: pt,
            peer_bgp_id: Ipv4Addr::new(192, 0, 2, 1),
            peer_ip: IpAddr::from([192, 0, 2, 1]),
            peer_asn: BgpKitAsn::new_32bit(65535),
        })
    }
}

#[derive(Debug)]
pub struct PeerTable {
    peer_table: HashMap<u16, Peer>,
}

impl PeerTable {
    pub fn new(peer_table: HashMap<u16, Peer>) -> Self {
        Self { peer_table }
    }

    pub fn from(peer_table: &HashMap<u16, BgpKitPeer>) -> Self {
        let mut pt = HashMap::<u16, Peer>::new();
        for key in peer_table.keys() {
            pt.insert(*key, Peer::new(*peer_table.get(key).unwrap()));
        }
        Self::new(pt)
    }

    pub fn get_peer(&self, id: &u16) -> &Peer {
        self.peer_table.get(id).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_peer_new() {
        let bgpkit_peer = BgpKitPeer {
            peer_type: bgpkit_parser::models::PeerType::empty(),
            peer_bgp_id: Ipv4Addr::new(10, 0, 0, 1),
            peer_ip: IpAddr::from([10, 0, 0, 1]),
            peer_asn: BgpKitAsn::new_32bit(64512),
        };

        let peer = Peer::new(bgpkit_peer);
        assert_eq!(peer.0.peer_type, bgpkit_peer.peer_type);
        assert_eq!(peer.0.peer_bgp_id, bgpkit_peer.peer_bgp_id);
        assert_eq!(peer.0.peer_ip, bgpkit_peer.peer_ip);
        assert_eq!(peer.0.peer_asn, bgpkit_peer.peer_asn);
    }

    #[test]
    fn test_peer_get_mock() {
        let peer = Peer::get_mock();

        assert_eq!(peer.0.peer_bgp_id, Ipv4Addr::new(192, 0, 2, 1));
        assert_eq!(peer.0.peer_ip, IpAddr::from([192, 0, 2, 1]));
        assert_eq!(peer.0.peer_asn.to_u32(), 65535);
        assert!(
            peer.0
                .peer_type
                .contains(bgpkit_parser::models::PeerType::AS_SIZE_32BIT)
        );
    }

    #[test]
    fn test_peer_serialize() {
        let test_cases = vec![
            (Ipv4Addr::new(1, 2, 3, 4), IpAddr::from([1, 2, 3, 4]), 100),
            (
                Ipv4Addr::new(10, 20, 30, 40),
                IpAddr::from([10, 20, 30, 40]),
                64512,
            ),
            (
                Ipv4Addr::new(255, 255, 255, 255),
                IpAddr::from([255, 255, 255, 255]),
                4294967295,
            ),
        ];

        for (bgp_id, ip, asn) in test_cases {
            let peer = Peer::new(BgpKitPeer {
                peer_type: bgpkit_parser::models::PeerType::empty(),
                peer_bgp_id: bgp_id,
                peer_ip: ip,
                peer_asn: BgpKitAsn::new_32bit(asn),
            });

            let json = serde_json::to_string(&peer).unwrap();
            assert!(json.contains("\"peer_bgp_id\""));
            assert!(json.contains("\"peer_ip\""));
            assert!(json.contains("\"peer_asn\""));
            assert!(json.contains(bgp_id.to_string().as_str()));
            assert!(json.contains(ip.to_string().as_str()));
            assert!(json.contains(&asn.to_string()));
        }
    }

    #[test]
    fn test_peer_clone() {
        let peer1 = Peer::get_mock();
        let peer2 = peer1.clone();
        assert_eq!(peer1, peer2);
    }

    #[test]
    fn test_peer_partial_eq() {
        let peer1 = Peer::get_mock();
        let peer2 = Peer::get_mock();

        assert_eq!(peer1, peer2);

        let peer3 = Peer::new(BgpKitPeer {
            peer_type: bgpkit_parser::models::PeerType::empty(),
            peer_bgp_id: Ipv4Addr::new(10, 0, 0, 1),
            peer_ip: IpAddr::from([10, 0, 0, 1]),
            peer_asn: BgpKitAsn::new_32bit(64512),
        });

        assert_ne!(peer1, peer3);
    }

    #[test]
    fn test_peer_hash() {
        let peer1 = Peer::get_mock();
        let peer2 = Peer::get_mock();

        let mut set = HashSet::new();
        set.insert(peer1.clone());

        assert!(set.contains(&peer2));

        let peer3 = Peer::new(BgpKitPeer {
            peer_type: bgpkit_parser::models::PeerType::empty(),
            peer_bgp_id: Ipv4Addr::new(10, 0, 0, 1),
            peer_ip: IpAddr::from([10, 0, 0, 1]),
            peer_asn: BgpKitAsn::new_32bit(64512),
        });

        assert!(!set.contains(&peer3));
    }

    #[test]
    fn test_peer_debug() {
        let peer = Peer::get_mock();
        let debug_str = format!("{:?}", peer);
        assert!(
            debug_str
                == "Peer(Peer { peer_type: PeerType(AS_SIZE_32BIT), peer_bgp_id: 192.0.2.1, peer_ip: 192.0.2.1, peer_asn: 65535 })"
        );
    }

    #[test]
    fn test_peer_table_new() {
        let mut peer_map = HashMap::new();
        peer_map.insert(1, Peer::get_mock());

        let peer_table = PeerTable::new(peer_map.clone());

        assert_eq!(peer_table.peer_table.len(), 1);
        assert!(peer_table.peer_table.contains_key(&1));
    }

    #[test]
    fn test_peer_table_new_empty() {
        let peer_map = HashMap::new();
        let peer_table = PeerTable::new(peer_map);

        assert_eq!(peer_table.peer_table.len(), 0);
    }

    #[test]
    fn test_peer_table_new_multiple_peers() {
        let mut peer_map = HashMap::new();
        peer_map.insert(1, Peer::get_mock());
        peer_map.insert(2, Peer::get_mock());
        peer_map.insert(3, Peer::get_mock());

        let peer_table = PeerTable::new(peer_map);

        assert_eq!(peer_table.peer_table.len(), 3);
    }

    #[test]
    fn test_peer_table_from() {
        let mut bgpkit_peer_map = HashMap::new();

        let peer1 = BgpKitPeer {
            peer_type: bgpkit_parser::models::PeerType::empty(),
            peer_bgp_id: Ipv4Addr::new(10, 0, 0, 1),
            peer_ip: IpAddr::from([10, 0, 0, 1]),
            peer_asn: BgpKitAsn::new_32bit(64512),
        };

        let peer2 = BgpKitPeer {
            peer_type: bgpkit_parser::models::PeerType::empty(),
            peer_bgp_id: Ipv4Addr::new(10, 0, 0, 2),
            peer_ip: IpAddr::from([10, 0, 0, 2]),
            peer_asn: BgpKitAsn::new_32bit(64513),
        };

        bgpkit_peer_map.insert(1, peer1);
        bgpkit_peer_map.insert(2, peer2);

        let peer_table = PeerTable::from(&bgpkit_peer_map);

        assert_eq!(peer_table.peer_table.len(), 2);
        assert!(peer_table.peer_table.contains_key(&1));
        assert!(peer_table.peer_table.contains_key(&2));

        assert_eq!(peer_table.get_peer(&1).0.peer_asn.to_u32(), 64512);
        assert_eq!(peer_table.get_peer(&2).0.peer_asn.to_u32(), 64513);
    }

    #[test]
    fn test_peer_table_from_empty() {
        let bgpkit_peer_map: HashMap<u16, BgpKitPeer> = HashMap::new();
        let peer_table = PeerTable::from(&bgpkit_peer_map);

        assert_eq!(peer_table.peer_table.len(), 0);
    }

    #[test]
    fn test_peer_table_get_peer() {
        let mut peer_map = HashMap::new();
        let mock_peer = Peer::get_mock();
        peer_map.insert(1, mock_peer.clone());

        let peer_table = PeerTable::new(peer_map);
        let retrieved_peer = peer_table.get_peer(&1);

        assert_eq!(retrieved_peer, &mock_peer);
    }

    #[test]
    fn test_peer_table_get_peer_nonexistent_panics() {
        assert!(
            std::panic::catch_unwind(|| {
                let peer_map = HashMap::new();
                let peer_table = PeerTable::new(peer_map);
                peer_table.get_peer(&999);
            })
            .is_err()
        );
    }
}
