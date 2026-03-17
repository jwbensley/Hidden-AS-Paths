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
