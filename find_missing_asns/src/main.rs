use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

use clap::Parser;
use log::{debug, info};
use serde::Serialize;
use serde_json::Value;

const TIER1_ASNS: &[u32] = &[
    174, 701, 1273, 1299, 2914, 3257, 3320, 3356, 3491, 5511, 6453, 6461, 6762, 6830, 6939, 7018,
    12956,
];

type AsPaths = HashMap<String, Value>;
type IrrAsns = HashMap<u32, Vec<u32>>;

#[derive(Parser, Debug)]
#[command(
    version,
    about = "Scan AS Paths looking for missing ASNs based on IRR data and communities",
    long_about = None
)]
struct CliArgs {
    /// Enable debug logging
    #[arg(short, long, default_value_t = false)]
    debug: bool,

    /// Path to the JSON file containing AS paths
    #[arg(short, long, default_value = "results/as_paths.json")]
    aspaths: PathBuf,

    /// Path to the JSON file containing IRR AS data
    #[arg(short, long, default_value = "results/irr_asns.json")]
    irr: PathBuf,

    /// Path to the JSON file containing IXP RS ASNs
    #[arg(short = 'x', long, default_value = "results/ixp_rs_asns.json")]
    ixprs: PathBuf,

    /// Directory to save the output JSON file
    #[arg(short, long, default_value = "results/")]
    output: PathBuf,
}

fn load_aspaths(filename: &PathBuf) -> AsPaths {
    let content = fs::read_to_string(filename).expect("Failed to read AS paths file");
    let data: Value = serde_json::from_str(&content).expect("Failed to parse AS paths JSON");
    let paths = data["paths"]
        .as_object()
        .expect("Expected 'paths' object")
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect::<AsPaths>();
    info!("Loaded {} AS paths", paths.len());
    paths
}

fn load_irr_asns(filename: &PathBuf) -> IrrAsns {
    let content = fs::read_to_string(filename).expect("Failed to read IRR ASNs file");
    let data: HashMap<String, Vec<u32>> =
        serde_json::from_str(&content).expect("Failed to parse IRR ASNs JSON");
    let result: IrrAsns = data
        .into_iter()
        .map(|(k, v)| {
            (
                k.parse::<u32>()
                    .unwrap_or_else(|_| panic!("Invalid ASN key {}", k)),
                v,
            )
        })
        .collect();
    info!("Loaded IRR ASNs for {} ASNs", result.len());
    result
}

fn load_ixp_rs_asns(filename: &PathBuf) -> HashSet<u32> {
    let content = fs::read_to_string(filename).expect("Failed to read IXP RS ASNs file");
    let data: Vec<u32> = serde_json::from_str(&content).expect("Failed to parse IXP RS ASNs JSON");
    let result: HashSet<u32> = data.into_iter().collect();
    info!("Loaded {} IXP RS ASNs", result.len());
    result
}

fn check_communities(route: &Value, asns: &[u32]) -> Vec<u32> {
    let mut community_asns: Vec<u32> = vec![];
    let communities = match route["communities"].as_array() {
        Some(c) => c,
        None => return community_asns,
    };

    let prefix = route["prefix"].as_str().unwrap_or_else(|| {
        panic!("Unable to unpack prefix from route: {:#?}", route);
    });

    for &asn in asns {
        for community in communities {
            let parts = community.as_array();
            if let Some(parts) = parts {
                let c0 = parts.first().and_then(|v| v.as_u64()).map(|v| v as u32);
                let c1 = parts.get(1).and_then(|v| v.as_u64()).map(|v| v as u32);
                if c0 == Some(asn) || c1 == Some(asn) {
                    community_asns.push(asn);
                    debug!(
                        "Community {:?} indicates prefix {} might be via AS{}",
                        community, prefix, asn
                    );
                    break;
                }
            }
        }
    }

    if community_asns.is_empty() {
        debug!(
            "No communities indicate prefix {} is via ASNs {:?}",
            prefix, asns
        );
    }
    community_asns
}

fn check_irr_for_asn(irr_asns: &IrrAsns, current_asn: u32, next_asn: u32) -> Vec<u32> {
    let mut peer_asns: Vec<u32> = vec![];

    if let Some(members) = irr_asns.get(&current_asn) {
        for &asn in members {
            if let Some(asn_members) = irr_asns.get(&asn) {
                if asn_members.contains(&next_asn) {
                    debug!("AS{} found in AS-SET of AS{}", next_asn, asn);
                    peer_asns.push(asn);
                }
            } else {
                debug!("AS{} does not have an AS-SET in IRR data, skipping", asn);
            }
        }
    }

    debug!(
        "AS{} is missing from AS-SET of AS{} and its members",
        next_asn, current_asn
    );
    peer_asns
}

fn find_missing_asns(
    aspaths: &AsPaths,
    irr_asns: &IrrAsns,
    ixp_rs_asns: &HashSet<u32>,
) -> HashMap<u32, HashMap<String, Value>> {
    let mut count = 0;
    let mut candidate_paths: HashMap<u32, HashMap<String, Value>> = HashMap::new();

    for (as_path, route) in aspaths {
        debug!("Processing AS path: {}", as_path);
        count += 1;
        if count % 100000 == 0 {
            info!("Processed {} AS paths", count);
        }

        let asns: Vec<u32> = as_path
            .split(',')
            .filter_map(|s| s.trim().parse::<u32>().ok())
            .collect();

        for i in 0..asns.len().saturating_sub(1) {
            let current_asn = asns[i];
            let next_asn = asns[i + 1];

            if TIER1_ASNS.contains(&current_asn) {
                debug!("Skipping AS{} in Tier 1 AS list", current_asn);
                continue;
            }

            if ixp_rs_asns.contains(&current_asn) {
                debug!("Skipping AS{} in IXP RS list", current_asn);
                continue;
            }

            if let Some(members) = irr_asns.get(&current_asn)
                && members.contains(&next_asn)
            {
                debug!("AS{} present in AS-SET of AS{}", next_asn, current_asn);
                continue;
            }

            if i + 2 == asns.len() {
                continue;
            }

            let via_peer_asns = check_irr_for_asn(irr_asns, current_asn, next_asn);
            if via_peer_asns.is_empty() {
                debug!(
                    "AS{} is missing from AS-SET of AS{} and its peers",
                    next_asn, current_asn
                );
                continue;
            }

            let community_asns = check_communities(route, &via_peer_asns);
            if !community_asns.is_empty() {
                debug!(
                    "Adding candidate path: {}. {} -> {} could be via {:?}",
                    as_path, current_asn, next_asn, community_asns
                );
                let mut enriched_route = route.clone();
                if let Some(obj) = enriched_route.as_object_mut() {
                    obj.insert("current_asn".to_string(), Value::from(current_asn));
                    obj.insert("next_asn".to_string(), Value::from(next_asn));
                    obj.insert(
                        "via_peer_asns".to_string(),
                        serde_json::to_value(&via_peer_asns).unwrap(),
                    );
                    obj.insert(
                        "community_asns".to_string(),
                        serde_json::to_value(&community_asns).unwrap(),
                    );
                }
                candidate_paths
                    .entry(current_asn)
                    .or_default()
                    .insert(as_path.clone(), enriched_route);
            }
        }
    }

    let total: usize = candidate_paths.values().map(|v| v.len()).sum();
    info!("Found {} candidate paths with missing ASNs", total);
    candidate_paths
}

fn parse_args() -> CliArgs {
    CliArgs::parse()
}

fn setup_logging(debug: bool) {
    let log_level = if debug { "debug" } else { "info" };
    env_logger::Builder::new()
        .parse_filters(log_level)
        .format(|buf, record| {
            use std::io::Write;
            writeln!(
                buf,
                "{}|{}|{}|{}|{}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                record.level(),
                std::process::id(),
                record.target(),
                record.args()
            )
        })
        .init();
}

fn write_json<T: Serialize>(data: &T, filename: &PathBuf) {
    let content = serde_json::to_string_pretty(data).expect("Failed to serialize JSON");
    fs::write(filename, content).expect("Failed to write output file");
    info!("Wrote output to {}", filename.display());
}

fn main() {
    let args = parse_args();
    setup_logging(args.debug);

    let aspaths = load_aspaths(&args.aspaths);
    let irr_asns = load_irr_asns(&args.irr);
    let ixp_rs_asns = load_ixp_rs_asns(&args.ixprs);
    let candidate_paths = find_missing_asns(&aspaths, &irr_asns, &ixp_rs_asns);

    let output_file = args.output.join("missing_asns.json");
    write_json(&candidate_paths, &output_file);
}
