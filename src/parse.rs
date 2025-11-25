pub mod parse {
    use crate::ribs::ribs::RibFile;
    use bgpkit_parser::BgpElem;
    use bgpkit_parser::BgpkitParser;
    use bgpkit_parser::MrtRecord;
    use bgpkit_parser::models::Asn;
    use bgpkit_parser::models::elem;
    use ipnet::IpNet;
    use log::info;
    use std::fmt::format;
    use std::net::IpAddr;
    use std::{collections::HashMap, path::Path};

    #[derive(Debug)]
    struct Route {
        prefix: IpNet,
        aspath: Vec<Asn>,
        aspath_deduped: Vec<Asn>,
        peer_ip: IpAddr,
        peer_asn: Asn,
        next_hop: IpAddr,
        filename: String,
    }

    pub fn parse_ribs(dir: &str, rib_files: &Vec<RibFile>) {
        let paths = HashMap::<Vec<u32>, Route>::new();

        for rib_file in rib_files {
            let mut count = 0;
            let fp = Path::new(dir)
                .join(rib_file.filename.as_str())
                .into_os_string()
                .into_string()
                .unwrap();

            info!("Parsing {}", fp);
            let parser =
                BgpkitParser::new(fp.as_str()).expect(format!("Unable to parse {}", fp).as_str());

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

            // let elem: MrtRecord;
            for elem in parser.into_record_iter() {
                println!("{:?}\n", elem);
                count += 1;
                if count == 3 {
                    break;
                }
            }
            break;
        }
    }
}
