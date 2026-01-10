pub mod rib_parser {
    use crate::mrt_paths::path_data::PathData;
    use crate::mrt_route::route::Route;
    use crate::ribs::rib_getter::RibFile;
    use bgpkit_parser::models::{
        AsPathSegment, Asn, AttrFlags, AttrType, Attribute, AttributeValue, Community,
        LargeCommunity, MrtMessage, Peer, RibAfiEntries, RibEntry, TableDumpV2Message,
        TableDumpV2Type,
    };
    use bgpkit_parser::{BgpkitParser, MrtRecord};
    use ipnet::IpNet;
    use log::{debug, info};
    use rayon::ThreadPoolBuilder;
    use rayon::iter::{IntoParallelIterator, ParallelIterator};
    use std::collections::HashMap;
    use std::net::IpAddr;

    /// Given a list of RIB files, parse them, merge the results, and strip and redundant data
    pub fn get_path_data(rib_files: &Vec<RibFile>, threads: &u32) -> PathData {
        let all_mrts_path_data = parse_rib_files(rib_files, threads);
        let mut merged_path_data = PathData::merge_path_data(all_mrts_path_data);
        merged_path_data.remove_single_hop_as_paths();
        merged_path_data.remove_origins_with_single_as_path();
        merged_path_data
    }

    /// Spin up a separate tread for each MRT file which needs to be parsed
    pub fn parse_rib_files(rib_files: &Vec<RibFile>, threads: &u32) -> Vec<PathData> {
        info!("Paring {} RIB files", rib_files.len());
        debug!(
            "{:?}",
            rib_files
                .iter()
                .map(|x| &x.filename)
                .collect::<Vec<&String>>()
        );

        ThreadPoolBuilder::new()
            .num_threads((*threads).try_into().unwrap())
            .build_global()
            .unwrap();

        let path_data = rib_files
            .into_par_iter()
            .map(|rib_file| parse_rib_file(rib_file.filename.clone()))
            .collect();

        info!("All RIB files parse");
        path_data
    }

    /// Return the mapping of peer IDs to peer details
    fn get_peer_id_map(fp: &String) -> HashMap<u16, Peer> {
        let parser = BgpkitParser::new(fp.as_str())
            .unwrap_or_else(|_| panic!("Unable to parse MRT file {}", fp));

        let mrt_record = parser.into_record_iter().next().unwrap();

        if let MrtMessage::TableDumpV2Message(TableDumpV2Message::PeerIndexTable(peer_table)) =
            &mrt_record.message
        {
            peer_table.id_peer_map.clone()
        } else {
            panic!("Couldn't extract peer table from table dump in {}", fp);
        }
    }

    /// Return the RIB entry in the MRT record.
    /// This is either a single v4 prefix or a single v6 prefix
    /// Skip default route.
    fn get_rib_entries<'a>(
        mrt_entry: &'a MrtRecord,
        fp: &String,
        count: &u32,
    ) -> Option<&'a RibAfiEntries> {
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
                    "Unexpected record type {:#?} in file {} ({})",
                    mrt_entry.message, fp, count
                ),
            }
        } else {
            panic!(
                "MRT record isn't of type RibAfi in file {} ({}): {:#?}",
                fp, count, mrt_entry
            );
        }
    }

    /// Return the next-nop which can be v4 or v6.
    /// If v6 LL and GUA nh exists, GUA is returned.
    fn get_next_hop(rib_entry: &RibEntry, fp: &String, count: &u32) -> IpAddr {
        if rib_entry.attributes.get_reachable_nlri().is_some() {
            let mp_nlri = rib_entry
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

            mp_nlri.next_hop_addr()
        } else {
            rib_entry.attributes.next_hop().unwrap_or_else(|| {
                panic!(
                    "No next-hop in file {} ({}) for: {:#?}",
                    fp, count, rib_entry
                )
            })
        }
    }

    fn get_communities(rib_entry: &RibEntry) -> Vec<Community> {
        if let AttributeValue::Communities(communities) = rib_entry
            .attributes
            .get_attr(AttrType::COMMUNITIES)
            .unwrap_or(Attribute {
                value: AttributeValue::Communities(Vec::new()),
                flag: AttrFlags::OPTIONAL | AttrFlags::TRANSITIVE,
            })
            .value
        {
            communities
        } else {
            Vec::new()
        }
    }

    fn get_large_communities(rib_entry: &RibEntry) -> Vec<LargeCommunity> {
        if let AttributeValue::LargeCommunities(large_communities) = rib_entry
            .attributes
            .get_attr(AttrType::LARGE_COMMUNITIES)
            .unwrap_or(Attribute {
                value: AttributeValue::LargeCommunities(Vec::new()),
                flag: AttrFlags::OPTIONAL | AttrFlags::TRANSITIVE,
            })
            .value
        {
            large_communities
        } else {
            Vec::new()
        }
    }

    /// Split the segments of the AS Path into an AS Sequence and an AS Set.
    /// The likelihood of there being more than on AS Sequnece (because the path)
    /// is longer than 255 ASNs is incredibly low. Equally the likely of more than
    /// one AS Set being present is incredily low. So we make the lazy assumption
    /// in the DFZ we'll see one of each, or one of both.
    fn get_as_path_chunks(rib_entry: &RibEntry, fp: &String, count: &u32) -> (Vec<Asn>, Vec<Asn>) {
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

        let mut as_sequence = Vec::<Asn>::new();
        let mut as_set = Vec::<Asn>::new();

        for path_seg in as_path_segments {
            if let AsPathSegment::AsSequence(asns) = path_seg {
                as_sequence = asns.clone();
            } else if let AsPathSegment::AsSet(asns) = path_seg {
                as_set = asns.clone();
            } else {
                panic!(
                    "Couldn't extract AS path sequence in file {} ({}): {:#?}",
                    fp, count, path_seg
                );
            }
        }

        if as_sequence.is_empty() && !as_set.is_empty() {
            panic!(
                "AS set defined without an AS sequence in file {} ({}): {:#?}",
                fp, count, rib_entry
            );
        }

        (as_sequence, as_set)
    }

    fn parse_rib_entries(
        mrt_entry: &MrtRecord,
        path_data: &mut PathData,
        id_peer_map: &HashMap<u16, Peer>,
        fp: &String,
        count: &u32,
    ) {
        let rib_entries = get_rib_entries(mrt_entry, fp, count);
        if rib_entries.is_none() {
            return;
        }
        let rib_entries = rib_entries.unwrap_or_else(|| {
            panic!(
                "Unable to consume RIB entries from {}: {:#?}",
                fp, mrt_entry
            )
        });

        for rib_entry in &rib_entries.rib_entries {
            let next_hop = get_next_hop(rib_entry, fp, count);
            //////// let communities = get_communities(rib_entry);
            //////// let large_communities = get_large_communities(rib_entry);

            // Split each AS path segment into an AsSequence and AsSet.
            // If an AsSet is defined, for each ASN in the set, create a unique AS path
            // (the AS Sequence + the AsSet ASN) and record the prefix as being available
            // via multiple AS Paths.
            let (as_sequence, as_set) = get_as_path_chunks(rib_entry, fp, count);

            if as_sequence.is_empty() && as_set.is_empty() {
                debug!(
                    "Skipping empty AS sequence and empty AS set, assuming iBGP path in file {} ({}): {:#?}",
                    fp, count, rib_entry
                );
                continue;
            }

            if !as_set.is_empty() {
                for asn in &as_set {
                    let mut as_path = as_sequence.clone();
                    as_path.push(*asn);

                    path_data.insert_route(Route::new(
                        as_path.clone(),
                        fp.clone(),
                        next_hop,
                        id_peer_map[&rib_entry.peer_index],
                        rib_entries.prefix.prefix,
                        // communities.clone(),
                        // large_communities.clone(),
                    ));
                }
            } else {
                path_data.insert_route(Route::new(
                    as_sequence.clone(),
                    fp.clone(),
                    next_hop,
                    id_peer_map[&rib_entry.peer_index],
                    rib_entries.prefix.prefix,
                    // communities.clone(),
                    // large_communities.clone(),
                ));
            }
        }
    }

    fn parse_rib_file(fp: String) -> PathData {
        info!("Parsing {}", fp);

        let mut path_data = PathData::new();
        let mut count: u32 = 0;
        let mut id_peer_map = HashMap::<u16, Peer>::new();

        let parser =
            BgpkitParser::new(fp.as_str()).unwrap_or_else(|_| panic!("Unable to parse {}", fp));

        for mrt_entry in parser.into_record_iter() {
            if count == 0 {
                id_peer_map = get_peer_id_map(&fp);
                debug!("Peer Map: {:#?}\n", id_peer_map);
                count += 1;
                continue;
            }

            parse_rib_entries(&mrt_entry, &mut path_data, &id_peer_map, &fp, &count);

            count += 1;
        }

        info!(
            "Parsed {} records in {}. Found {} origins with {} AS paths.",
            count,
            fp,
            path_data.get_origins_count(),
            path_data.get_as_paths_count(),
        );

        path_data
    }
}
