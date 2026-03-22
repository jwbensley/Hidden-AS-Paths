use crate::data::paths::Paths;
use crate::data::record_data::RecordData;
use crate::types::as_path::{self, AsPath};
use crate::types::asn::Asn;
use crate::types::community::StandardCommunity;
use crate::types::peer::PeerTable;
use crate::types::route::Route;
use bgpkit_parser::models::{
    AsPathSegment, AttrFlags, AttrType, Attribute, AttributeValue, MrtMessage, RibAfiEntries,
    RibEntry, TableDumpV2Message, TableDumpV2Type,
};
use bgpkit_parser::{BgpkitParser, MrtRecord};
use ipnet::IpNet;
use std::net::IpAddr;
use std::sync::LazyLock;

static V4_DEFAULT: LazyLock<IpNet> = LazyLock::new(|| "0.0.0.0/0".parse().unwrap());
static V6_DEFAULT: LazyLock<IpNet> = LazyLock::new(|| "::/0".parse().unwrap());

/// Return the mapping of peer IDs to peer details
pub fn get_peer_table(fp: &String) -> PeerTable {
    let parser = BgpkitParser::new(fp.as_str())
        .unwrap_or_else(|_| panic!("Unable to parse MRT file {}", fp));

    let mrt_record = parser
        .into_record_iter()
        .next()
        .unwrap_or_else(|| panic!("Unable to extract first record from {}", fp));

    if let MrtMessage::TableDumpV2Message(TableDumpV2Message::PeerIndexTable(peer_table)) =
        &mrt_record.message
    {
        PeerTable::from(&peer_table.id_peer_map)
    } else {
        panic!("Couldn't extract peer table from table dump in {}", fp);
    }
}

/// Return the RIB entry in the MRT record.
/// This is either a single v4 prefix or a single v6 prefix
/// Skip default route.
fn get_rib_entries<'a>(mrt_entry: &'a MrtRecord, fp: &String) -> Option<&'a RibAfiEntries> {
    if let MrtMessage::TableDumpV2Message(TableDumpV2Message::RibAfi(rib_entries)) =
        &mrt_entry.message
    {
        match rib_entries.rib_type {
            TableDumpV2Type::RibIpv4Unicast | TableDumpV2Type::RibIpv4UnicastAddPath => {
                if rib_entries.prefix.prefix == *V4_DEFAULT {
                    return None;
                }
                Some(rib_entries)
            }
            TableDumpV2Type::RibIpv6Unicast | TableDumpV2Type::RibIpv6UnicastAddPath => {
                if rib_entries.prefix.prefix == *V6_DEFAULT {
                    return None;
                }
                Some(rib_entries)
            }
            _ => panic!(
                "Unexpected record type {:#?} in file {}",
                mrt_entry.message, fp
            ),
        }
    } else {
        panic!(
            "MRT record isn't of type RibAfi in file {}: {:#?}",
            fp, mrt_entry
        );
    }
}

/// For a given mrt record, extract the prefix, then parse all rib entries for that prefix.
pub fn parse_mrt_entry(mrt_data: RecordData) {
    let rib_entries = get_rib_entries(mrt_data.mrt_record, mrt_data.mrt_fp);
    if rib_entries.is_none() {
        return;
    }
    let rib_entries = rib_entries.unwrap_or_else(|| {
        panic!(
            "Unable to consume RIB entries from {}: {:#?}",
            mrt_data.mrt_fp, mrt_data.mrt_record
        )
    });

    let prefix = rib_entries.prefix.prefix;

    for rib_entry in &rib_entries.rib_entries {
        parse_rib_entry(prefix, rib_entry, &mrt_data);
    }
}

/// Split the segments of the AS Path into an AS Sequence and an AS Set.
/// The likelihood of there being more than on AS Sequence because the path
/// is longer than 255 ASNs is incredibly low.
/// Also, we're not interested in AS_SETs because they are deprecated.
fn get_as_sequence(rib_entry: &RibEntry, fp: &String) -> AsPath {
    let as_path_segments = &rib_entry
        .attributes
        .as_path()
        .unwrap_or_else(|| {
            panic!(
                "Unable to unpack AS Path segments from RIB entry in {}:  {:#?}",
                fp, rib_entry
            )
        })
        .segments;

    for path_seg in as_path_segments {
        if let AsPathSegment::AsSequence(asns) = path_seg {
            return AsPath::from_vec(asns);
        }
    }

    AsPath::default()
}

/// Return the next-nop which can be v4 or v6.
/// If v6 LL and GUA nh exists, GUA is returned.
fn get_next_hop(rib_entry: &RibEntry, fp: &String) -> IpAddr {
    if rib_entry.attributes.get_reachable_nlri().is_some() {
        let mp_nlri = rib_entry
            .attributes
            .get_reachable_nlri()
            .unwrap_or_else(|| {
                panic!(
                    "Couldn't extract MP NLRI in file {} for: {:#?}",
                    fp, rib_entry
                )
            });

        assert!(
            mp_nlri.is_ipv6(),
            "MP NLRI is used for non-IPv6 info in file {} for: {:#?}",
            fp,
            rib_entry
        );

        mp_nlri.next_hop_addr()
    } else {
        rib_entry
            .attributes
            .next_hop()
            .unwrap_or_else(|| panic!("No next-hop in file {} for: {:#?}", fp, rib_entry))
    }
}

/// Get the list of standard communities
fn get_communities(rib_entry: &RibEntry) -> Vec<StandardCommunity> {
    if let AttributeValue::Communities(communities) = rib_entry
        .attributes
        .get_attr(AttrType::COMMUNITIES)
        .unwrap_or(Attribute {
            value: AttributeValue::Communities(Vec::new()),
            flag: AttrFlags::OPTIONAL | AttrFlags::TRANSITIVE,
        })
        .value
    {
        Vec::from_iter(communities.iter().map(StandardCommunity::from_community))
    } else {
        Vec::new()
    }
}

fn build_route(mrt_data: &RecordData, rib_entry: &RibEntry, prefix: &IpNet, origin: Asn) -> Route {
    let peer = mrt_data.peer_table.get_peer(&rib_entry.peer_index);
    let next_hop = get_next_hop(rib_entry, mrt_data.mrt_fp);
    let communities = get_communities(rib_entry);

    Route::new(
        origin,
        mrt_data.mrt_fp.clone(),
        next_hop,
        peer.to_owned(),
        prefix.to_owned(),
        communities,
    )
}

/// Extract the route from the RIB entry and the AS path for that route, then add the path
/// and route at the end of the path to the list of paths for the origin AS.
pub fn parse_rib_entry(prefix: IpNet, rib_entry: &RibEntry, mrt_data: &RecordData) {
    let as_sequence = get_as_sequence(rib_entry, mrt_data.mrt_fp);
    let origin = as_sequence.get_origin().clone();
    let route = build_route(mrt_data, rib_entry, &prefix, origin);

    if as_sequence.is_empty() {
        // Some collectors include iBGP paths or self originated prefixes with no AS path
        return;
    }

    let has_path: bool;
    {
        let paths: std::sync::RwLockReadGuard<'_, Paths> = mrt_data.paths.read().unwrap();
        has_path = paths.has_route(&route);
    }
    if !has_path {
        let mut paths = mrt_data.paths.write().unwrap();
        paths.add_route(route.clone(), &as_sequence);
    }
}
