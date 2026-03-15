pub mod mrt_parser {
    use crate::mrt_asn::asn::MrtAsn;
    use crate::mrt_communities::standard_communities::StandardCommunities;
    use crate::mrt_peer::peer::PeerTable;
    use crate::mrt_route::route::Route;
    use bgpkit_parser::models::{
        AsPathSegment, AttrFlags, AttrType, Attribute, AttributeValue, MrtMessage, RibAfiEntries,
        RibEntry, TableDumpV2Message, TableDumpV2Type,
    };
    use bgpkit_parser::{BgpkitParser, MrtRecord};
    use ipnet::IpNet;
    use std::net::IpAddr;
    use std::sync::{Arc, RwLock};

    // Shared data that needs to be passed around when parsing an MRT entry
    pub struct MrtData<'a> {
        mrt_entry: &'a MrtRecord,
        global_peerings: &'a Arc<RwLock<PeeringData>>,
        peer_id_map: &'a PeerTable,
        fp: &'a String,
    }

    impl<'a> MrtData<'a> {
        pub fn new(
            mrt_entry: &'a MrtRecord,
            global_peerings: &'a Arc<RwLock<PeeringData>>,
            peer_id_map: &'a PeerTable,
            fp: &'a String,
        ) -> Self {
            Self {
                mrt_entry,
                global_peerings,
                peer_id_map,
                fp,
            }
        }
    }

    /// Extract the prefix from each RIB entry and proceed to check the AS path for that prefix
    pub fn parse_mrt_entry(mrt_data: MrtData) {
        let rib_entries = get_rib_entries(mrt_data.mrt_entry, mrt_data.fp);
        if rib_entries.is_none() {
            return;
        }
        let rib_entries = rib_entries.unwrap_or_else(|| {
            panic!(
                "Unable to consume RIB entries from {}: {:#?}",
                mrt_data.fp, mrt_data.mrt_entry
            )
        });

        let prefix = rib_entries.prefix.prefix;

        for rib_entry in &rib_entries.rib_entries {
            check_as_seq(prefix, rib_entry, &mrt_data);
        }
    }

    /// Check an AS Path if it containers two neighboring T1 ASNs
    pub fn check_as_seq(prefix: IpNet, rib_entry: &RibEntry, mrt_data: &MrtData) {
        let mut as_sequence = get_as_sequence(rib_entry, mrt_data.fp);
        as_sequence.dedup();

        if as_sequence.is_empty() {
            // Some collectors include iBGP paths or self originated prefixes with no AS path
            return;
        }

        for asn_1 in as_sequence.iter() {
            // We could see up to three T1 ASNs in a row e.g. AS3 AS2 AS1 AS65535.
            // AS3 peers with AS2, AS1 is transit customer of AS2 (despite being "Tier 1").
            // AS65535 is non-T1 transit customer of AS1.
            // In this case we need to check AS3-AS2 communities and AS2-AS1 communities.
            let pos_1 = as_sequence.iter().position(|x| x == asn_1).unwrap();

            if pos_1 == as_sequence.len() - 1 {
                // Last ASN in the path
                break;
            }

            let pos_2 = pos_1 + 1;
            let asn_2 = &as_sequence[pos_2];
            let route = build_route(mrt_data, rib_entry, asn_1, asn_2, &as_sequence, &prefix);

            add_peering(mrt_data.global_peerings, &route);

            if pos_2 == as_sequence.len() - 1 {
                // Last ASN in the path
                break;
            }

            let asn_3 = &as_sequence[pos_2 + 1];
            let route = build_route(mrt_data, rib_entry, asn_2, asn_3, &as_sequence, &prefix);
            add_peering(mrt_data.global_peerings, &route);
        }
    }

    /// Return the mapping of peer IDs to peer details
    pub fn get_peer_id_map(fp: &String) -> PeerTable {
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
        let v4_default: IpNet = "0.0.0.0/0".parse().unwrap();
        let v6_default: IpNet = "::/0".parse().unwrap();

        if let MrtMessage::TableDumpV2Message(TableDumpV2Message::RibAfi(rib_entries)) =
            &mrt_entry.message
        {
            match rib_entries.rib_type {
                TableDumpV2Type::RibIpv4Unicast | TableDumpV2Type::RibIpv4UnicastAddPath => {
                    if rib_entries.prefix.prefix == v4_default {
                        return None;
                    }
                    Some(rib_entries)
                }
                TableDumpV2Type::RibIpv6Unicast | TableDumpV2Type::RibIpv6UnicastAddPath => {
                    if rib_entries.prefix.prefix == v6_default {
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

    fn get_communities(rib_entry: &RibEntry) -> StandardCommunities {
        if let AttributeValue::Communities(communities) = rib_entry
            .attributes
            .get_attr(AttrType::COMMUNITIES)
            .unwrap_or(Attribute {
                value: AttributeValue::Communities(Vec::new()),
                flag: AttrFlags::OPTIONAL | AttrFlags::TRANSITIVE,
            })
            .value
        {
            StandardCommunities::from_vec(communities)
        } else {
            StandardCommunities::from_vec(Vec::new())
        }
    }

    /// Split the segments of the AS Path into an AS Sequence and an AS Set.
    /// The likelihood of there being more than on AS Sequence (because the path)
    /// is longer than 255 ASNs is incredibly low. Also, because we're looking for
    /// T1 AS path, we're not interested in AS_SETs.
    fn get_as_sequence(rib_entry: &RibEntry, fp: &String) -> Vec<MrtAsn> {
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
                return asns
                    .iter()
                    .map(|a| MrtAsn::new(a.clone()))
                    .collect::<Vec<MrtAsn>>();
            }
        }

        Vec::<MrtAsn>::new()
    }

    fn build_route(
        mrt_data: &MrtData,
        rib_entry: &RibEntry,
        local_asn: &MrtAsn,
        peer_asn: &MrtAsn,
        as_sequence: &Vec<MrtAsn>,
        prefix: &IpNet,
    ) -> Route {
        let peer = mrt_data.peer_id_map.get_peer(&rib_entry.peer_index);
        let next_hop = get_next_hop(rib_entry, mrt_data.fp);
        let communities = get_communities(rib_entry);

        Route::new(
            local_asn.clone(),
            peer_asn.clone(),
            as_sequence.to_owned(),
            mrt_data.fp.clone(),
            next_hop.to_owned(),
            peer.to_owned(),
            prefix.to_owned(),
            communities.clone(),
        )
    }

    fn add_peering(global_peerings: &Arc<RwLock<PeeringData>>, route: &Route) {
        let has_peering: bool;
        {
            let lock = global_peerings.read().unwrap();
            has_peering = lock.has_peering(route);
        }
        if !has_peering {
            let mut lock = global_peerings.write().unwrap();
            lock.add_peering(route.clone());
        }
    }
}
