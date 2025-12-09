pub mod rib_parser {
    use crate::asp_tree::asp_trees::{AsSequences, Route};
    use crate::ribs::rib_getter::RibFile;
    use bgpkit_parser::BgpkitParser;
    use bgpkit_parser::models::{
        AsPathSegment, Asn, AttrFlags, AttrType, Attribute, AttributeValue, MrtMessage, Peer,
        TableDumpV2Message, TableDumpV2Type,
    };
    use core::panic;
    use ipnet::IpNet;
    use log::{debug, info};
    use std::net::{IpAddr, Ipv6Addr};
    use std::thread;
    use std::{collections::HashMap, path::Path};

    pub fn parse_ribs(dir: &str, rib_files: &Vec<RibFile>) -> Vec<AsSequences> {
        debug!(
            "Parsing {} RIB files: {:?}",
            rib_files.len(),
            rib_files
                .iter()
                .map(|x| &x.filename)
                .collect::<Vec<&String>>()
        );

        thread::scope(|s| {
            let mut handles = Vec::new();

            let mut i = 0; /////////////////////////////////////////////////////////////////////////////
            for rib_file in rib_files {
                if i >= 4 {
                    ///////////////////////////////////////////////////////////////////////////////////////////////////
                    break;
                }
                i += 1;

                let fp = Path::new(dir)
                    .join(rib_file.filename.as_str())
                    .into_os_string()
                    .into_string()
                    .unwrap();

                handles.push(s.spawn(|| parse_rib(fp)));
            }

            handles
                .into_iter()
                .map(|handle| handle.join().unwrap())
                .collect::<Vec<_>>()
        })
    }

    fn parse_rib(fp: String) -> AsSequences {
        let v4_default: IpNet = "0.0.0.0/0".parse().unwrap();
        let v6_default: IpNet = "::/0".parse().unwrap();
        let mut paths = AsSequences::new();
        let mut id_peer_map = HashMap::<u16, Peer>::new();
        let mut count = 0;

        info!("Parsing {}", fp);
        let parser =
            BgpkitParser::new(fp.as_str()).unwrap_or_else(|_| panic!("Unable to parse {}", fp));

        for elem in parser.into_record_iter() {
            if count == 0 {
                if let MrtMessage::TableDumpV2Message(TableDumpV2Message::PeerIndexTable(
                    peer_table,
                )) = &elem.message
                {
                    id_peer_map = peer_table.id_peer_map.clone();
                } else {
                    panic!("Couldn't extract peer table from table dump in {}", fp);
                }

                debug!("{:#?}\n", id_peer_map); ///////////////////////////////////////////////////////////////////////////////////////////////////
                count += 1;
                continue;
            }

            if let MrtMessage::TableDumpV2Message(TableDumpV2Message::RibAfi(rib_entries)) =
                &elem.message
            {
                match rib_entries.rib_type {
                    TableDumpV2Type::RibIpv4Unicast | TableDumpV2Type::RibIpv4UnicastAddPath => {
                        if rib_entries.prefix.prefix == v4_default {
                            continue;
                        }
                    }
                    TableDumpV2Type::RibIpv6Unicast | TableDumpV2Type::RibIpv6UnicastAddPath => {
                        if rib_entries.prefix.prefix == v6_default {
                            continue;
                        }
                    }
                    _ => panic!(
                        "Unexpected record type {:#?} in file {} ({})",
                        elem.message, fp, count
                    ),
                }

                for rib_entry in &rib_entries.rib_entries {
                    let as_path_segments = &rib_entry
                        .attributes
                        .as_path()
                        .unwrap_or_else(|| {
                            panic!(
                                "Unable to unpack AS Path segments from RIB entry {:#?}",
                                rib_entry
                            )
                        })
                        .segments;

                    let mut next_hop: IpAddr = IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0));

                    if rib_entry.attributes.get_reachable_nlri().is_some() {
                        let mp_nlri =
                            rib_entry
                                .attributes
                                .get_reachable_nlri()
                                .unwrap_or_else(|| {
                                    panic!(
                                        "Couldn't extract MP NLRI in file {} ({}) for: {:#?}",
                                        fp, count, rib_entry
                                    )
                                });

                        assert!(
                            mp_nlri.is_ipv6(),
                            "MP NLRI is used for non-IPv6 info in file {} ({}): {:#?}",
                            fp,
                            count,
                            rib_entry
                        );

                        next_hop = mp_nlri.next_hop_addr();
                    } else {
                        next_hop = rib_entry.attributes.next_hop().unwrap_or_else(|| {
                            panic!(
                                "No next-hop in file {} ({}) for: {:#?}",
                                fp, count, rib_entry
                            )
                        });
                    }

                    let mut communities = Vec::new();
                    let mut large_communities = Vec::new();

                    if let AttributeValue::Communities(c) = rib_entry
                        .attributes
                        .get_attr(AttrType::COMMUNITIES)
                        .unwrap_or(Attribute {
                            value: AttributeValue::Communities(Vec::new()),
                            flag: AttrFlags::OPTIONAL | AttrFlags::TRANSITIVE,
                        })
                        .value
                    {
                        communities = c;
                    }

                    if let AttributeValue::LargeCommunities(l) = rib_entry
                        .attributes
                        .get_attr(AttrType::LARGE_COMMUNITIES)
                        .unwrap_or(Attribute {
                            value: AttributeValue::LargeCommunities(Vec::new()),
                            flag: AttrFlags::OPTIONAL | AttrFlags::TRANSITIVE,
                        })
                        .value
                    {
                        large_communities = l;
                    }

                    let mut as_sequence = Vec::<Asn>::new();
                    let mut as_set = Vec::<Asn>::new();

                    for as_path_segment in as_path_segments {
                        if let AsPathSegment::AsSequence(asns) = as_path_segment {
                            as_sequence = asns.clone();
                        } else if let AsPathSegment::AsSet(asns) = as_path_segment {
                            as_set = asns.clone();
                        } else {
                            panic!(
                                "Couldn't extract AS path sequence in file {} ({}): {:#?}",
                                fp, count, rib_entry
                            );
                        }

                        if as_sequence.is_empty() {
                            if as_set.is_empty() {
                                panic!(
                                    "AS sequence and AS set are both undefined in file {} ({}): {:#?}",
                                    fp, count, rib_entry
                                );
                            } else {
                                panic!(
                                    "AS set defined without an AS sequence in file {} ({}): {:#?}",
                                    fp, count, rib_entry
                                );
                            }
                        }

                        if !as_set.is_empty() {
                            for asn in &as_set {
                                let mut as_path = as_sequence.clone();
                                as_path.push(asn.to_owned());

                                let mut deduped = as_path.clone();
                                deduped.dedup();

                                let route = Route::new(
                                    as_path.clone(),
                                    deduped.clone(),
                                    fp.clone(),
                                    next_hop,
                                    id_peer_map[&rib_entry.peer_index],
                                    rib_entries.prefix.prefix,
                                    communities.clone(),
                                    large_communities.clone(),
                                );
                                paths.insert_route_at_sequence(
                                    deduped.clone(),
                                    as_path.clone(),
                                    route,
                                );
                            }
                        } else {
                            let mut deduped = as_sequence.clone();
                            deduped.dedup();

                            let route = Route::new(
                                as_sequence.clone(),
                                deduped.clone(),
                                fp.clone(),
                                next_hop,
                                id_peer_map[&rib_entry.peer_index],
                                rib_entries.prefix.prefix,
                                communities.clone(),
                                large_communities.clone(),
                            );
                            paths.insert_route_at_sequence(deduped, as_sequence.clone(), route);
                        }
                    }
                }
            } else {
                panic!(
                    "MRT record isn't of type RibAfi in file {} ({}): {:#?}",
                    fp, count, elem
                );
            }

            count += 1;
            ///////////////////////////////////////////////////////////////////////////////////////////////////
            if count >= 2 {
                break;
            }
        }

        info!("Parsed {} records in MRT file", count);

        paths
    }
}

/*
MrtRecord {
    common_header: CommonHeader {
        timestamp: 1758499201,
        microsecond_timestamp: None,
        entry_type: TABLE_DUMP_V2,
        entry_subtype: 1,
        length: 1679
    },
    message: TableDumpV2Message(
        PeerIndexTable(
            PeerIndexTable {
                collector_bgp_id: 80.249.213.84,
                view_name: "VRF default",
                id_peer_map: {
                    76: Peer {
                        peer_type: PeerType(AS_SIZE_32BIT | ADDRESS_FAMILY_IPV6),
                        peer_bgp_id: 188.122.95.123,
                        peer_address: 2001:7f8:1::a504:9544:1,
                        peer_asn: 49544
                    },
                    38: Peer {
                        peer_type: PeerType(AS_SIZE_32BIT),
                        peer_bgp_id: 134.55.200.236,
                        peer_address: 80.249.213.7,
                        peer_asn: 293
                    },
                    ...
                }
            }
        )
    )
}

IPv4
MrtRecord {
    common_header: CommonHeader {
        timestamp: 1758499201,
        microsecond_timestamp: None,
        entry_type: TABLE_DUMP_V2,
        entry_subtype: 2,
        length: 80
    },
    message: TableDumpV2Message(
        RibAfi(
            RibAfiEntries {
                rib_type: RibIpv4Unicast,
                sequence_number: 0,
                prefix: 0.0.0.0/0,
                rib_entries: [
                    RibEntry {
                        peer_index: 41,
                        originated_time: 1756151844,
                        attributes: Attributes {
                            inner: [
                                Attribute {
                                    value: Origin(IGP),
                                    flag: AttrFlags(TRANSITIVE)
                                },
                                Attribute {
                                    value: AsPath {
                                        path: AsPath {
                                            segments: [
                                                AsSequence([64289, 6762])
                                            ]
                                        },
                                        is_as4: false
                                    },
                                    flag: AttrFlags(TRANSITIVE | EXTENDED)
                                },
                                Attribute {
                                    value: NextHop(80.249.214.73),
                                    flag: AttrFlags(TRANSITIVE)
                                },
                                Attribute {
                                    value: Communities([Custom(64289, 1000)]),
                                    flag: AttrFlags(OPTIONAL | TRANSITIVE)
                                }
                            ]
                        }
                    },
                    RibEntry { peer_index: 29, originated_time: 1757942513, attributes: Attributes { inner: [Attribute { value: Origin(IGP), flag: AttrFlags(TRANSITIVE) }, Attribute { value: AsPath { path: AsPath { segments: [AsSequence([61049, 3257])] }, is_as4: false }, flag: AttrFlags(TRANSITIVE | EXTENDED) }, Attribute { value: NextHop(80.249.210.252), flag: AttrFlags(TRANSITIVE) }] } }
                ]
            }
        )
    )
}

IPv6
Attribute {
  value: MpReachNlri(
    Nlri {
      afi: Ipv6,
      safi: Unicast,
      next_hop: Some(
        Ipv6LinkLocal(2001:7f8:1:0:a500:32:8832:1, fe80::4201:7aff:fe41:a186)
      ),
      prefixes: [200:1900:5203::/56]
    }
  ),
  flag: AttrFlags(OPTIONAL)
}


 */
