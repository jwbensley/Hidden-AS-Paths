pub mod parse {
    use crate::ribs::ribs::RibFile;
    use bgpkit_parser::BgpkitParser;
    use bgpkit_parser::models::AsPathSegment;
    use bgpkit_parser::models::Asn;
    use bgpkit_parser::models::MrtMessage;
    use bgpkit_parser::models::Peer;
    use bgpkit_parser::models::TableDumpV2Message;
    use core::panic;
    use ipnet::IpNet;
    use log::{debug, info};
    use std::net::IpAddr;
    use std::str::FromStr;
    use std::{collections::HashMap, path::Path};

    #[derive(Debug)]
    struct Route {
        aspath: Vec<Asn>,
        aspath_deduped: Vec<Asn>,
        filename: String,
        next_hop: IpAddr,
        peer: Peer,
        prefix: IpNet,
    }

    impl PartialEq for Route {
        fn eq(&self, other: &Self) -> bool {
            (self.aspath == other.aspath)
                && (self.aspath_deduped == other.aspath_deduped)
                && (self.filename == other.filename)
                && (self.next_hop == other.next_hop)
                && (self.peer == other.peer)
                && (self.prefix == other.prefix)
        }
    }

    #[derive(Debug)]
    struct Paths {
        paths: HashMap<Vec<Asn>, HashMap<Vec<Asn>, Vec<Route>>>,
    }

    impl Paths {
        pub fn new() -> Self {
            Paths {
                paths: HashMap::<Vec<Asn>, HashMap<Vec<Asn>, Vec<Route>>>::new(),
            }
        }

        fn has_deduped_path(&self, deduped_path: &Vec<Asn>) -> bool {
            debug!(
                "Deduped AS sequence present: {}, {:?}",
                self.paths.contains_key(deduped_path),
                deduped_path
            );
            self.paths.contains_key(deduped_path)
        }

        fn insert_deduped_path(&mut self, deduped_path: Vec<Asn>) {
            debug!("Adding deduped AS sequence: {:?}", deduped_path);
            self.paths.insert(deduped_path, HashMap::new());
        }

        fn has_path(&self, deduped_path: &Vec<Asn>, path: &Vec<Asn>) -> bool {
            debug!(
                "AS sequence present in {:?}: {}, {:?}",
                deduped_path,
                self.paths[deduped_path].contains_key(path),
                path
            );
            self.paths[deduped_path].contains_key(path)
        }

        fn insert_path(&mut self, deduped_path: &Vec<Asn>, path: Vec<Asn>) {
            debug!("Adding AS sequence in {:?}: {:?}", deduped_path, path);
            self.paths
                .get_mut(deduped_path)
                .unwrap()
                .insert(path, Vec::new());
        }

        fn has_route(&self, deduped_path: &Vec<Asn>, path: &Vec<Asn>, route: &Route) -> bool {
            debug!(
                "Route sequence present in {:?}, {:?}: {}, {:?}",
                deduped_path,
                path,
                self.paths[deduped_path][path].contains(route),
                route
            );
            self.paths[deduped_path][path].contains(route)
        }

        fn insert_route(&mut self, deduped_path: &Vec<Asn>, path: &Vec<Asn>, route: Route) {
            debug!(
                "Adding route in {:?}, {:?}: {:?}",
                deduped_path, path, route
            );
            self.paths
                .get_mut(deduped_path)
                .unwrap()
                .get_mut(path)
                .unwrap()
                .push(route);
        }

        fn insert_route_from_root(&mut self, deduped_path: Vec<Asn>, path: Vec<Asn>, route: Route) {
            if !self.has_deduped_path(&deduped_path) {
                self.insert_deduped_path(deduped_path.clone());
            }
            if !self.has_path(&deduped_path, &path) {
                self.insert_path(&deduped_path, path.clone());
            }
            if !self.has_route(&deduped_path, &path, &route) {
                self.insert_route(&deduped_path, &path, route);
            }
        }
    }

    pub fn parse_ribs(dir: &str, rib_files: &Vec<RibFile>) {
        let mut paths = Paths::new();

        let v4_default = IpNet::from_str("0.0.0.0/0").unwrap();
        let v6_default = IpNet::from_str("::/0").unwrap();

        for rib_file in rib_files {
            let fp = Path::new(dir)
                .join(rib_file.filename.as_str())
                .into_os_string()
                .into_string()
                .unwrap();

            let mut id_peer_map = HashMap::<u16, Peer>::new();
            let mut count = 0;

            info!("Parsing {}", fp);
            let parser =
                BgpkitParser::new(fp.as_str()).expect(format!("Unable to parse {}", fp).as_str());

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

                    debug!("{:?}\n", id_peer_map);
                    count += 1;
                    continue;
                }

                if let MrtMessage::TableDumpV2Message(TableDumpV2Message::RibAfi(rib_entries)) =
                    &elem.message
                {
                    match rib_entries.rib_type {
                        bgpkit_parser::models::TableDumpV2Type::RibIpv4Unicast
                        | bgpkit_parser::models::TableDumpV2Type::RibIpv4UnicastAddPath => {
                            if rib_entries.prefix.prefix == v4_default {
                                continue;
                            }
                        }
                        bgpkit_parser::models::TableDumpV2Type::RibIpv6Unicast
                        | bgpkit_parser::models::TableDumpV2Type::RibIpv6UnicastAddPath => {
                            if rib_entries.prefix.prefix == v6_default {
                                continue;
                            }
                        }
                        _ => panic!(
                            "Unexpected record type {:?} in file {} ({})",
                            elem.message, fp, count
                        ),
                    }

                    for rib_entry in &rib_entries.rib_entries {
                        let as_path_segments = &rib_entry
                            .attributes
                            .as_path()
                            .expect(
                                format!(
                                    "Unable to unpack AS Path segments from RIB entry {:?}",
                                    rib_entry
                                )
                                .as_str(),
                            )
                            .segments;

                        let next_hop = rib_entry.attributes.next_hop().expect(
                            format!(
                                "No next-hop in file {} ({}) for: {:?}",
                                fp, count, rib_entry
                            )
                            .as_str(),
                        );

                        for as_path_segment in as_path_segments {
                            if let AsPathSegment::AsSequence(as_sequence) = as_path_segment {
                                let mut deduped = as_sequence.clone();
                                deduped.dedup();

                                let route = Route {
                                    aspath: as_sequence.clone(),
                                    aspath_deduped: deduped.clone(),
                                    filename: fp.clone(),
                                    next_hop: next_hop.clone(),
                                    prefix: rib_entries.prefix.prefix.clone(),
                                    peer: id_peer_map[&rib_entry.peer_index].clone(),
                                };
                                paths.insert_route_from_root(deduped, as_sequence.clone(), route);
                            } else {
                                panic!(
                                    "Couldn't extract AS path sequence in file {} ({}): {:?}",
                                    fp, count, as_path_segment
                                );
                            }
                        }
                    }
                } else {
                    panic!(
                        "MRT record isn't of type RibAfi in file {} ({}): {:?}",
                        fp, count, elem
                    );
                }

                count += 1;
                // if count >= 5 {
                //     break;
                // }
            }

            info!("Parsed {} records in MRT file", count);

            break;
        }
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
 */
