#!/usr/bin/env -S uv run --script
#
# /// script
# requires-python = "==3.14"
# ///

import argparse
import json
import logging
import os
from typing import Any
from globals import TIER1_ASNS


def setup_logging(debug: bool) -> None:
    level = logging.DEBUG if debug else logging.INFO

    logging.basicConfig(
        format="%(asctime)s|%(levelname)s|%(process)d|%(funcName)s|%(message)s",
        level=level,
        handlers=[
            logging.StreamHandler(),
        ],
    )


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Scan AS Paths, for each ASN, check if the next ASN is in the AS-SET of the current ASN.",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    parser.add_argument(
        "--debug",
        "-d",
        action="store_true",
        help="Enable debug logging",
        default=False,
    )
    parser.add_argument(
        "--aspaths",
        "-a",
        type=str,
        default="results/as_paths.json",
        help="Path to the JSON file containing AS paths",
    )
    parser.add_argument(
        "--irr",
        "-i",
        type=str,
        default="results/irr_asns.json",
        help="Path to the JSON file containing IRR AS data",
    )
    parser.add_argument(
        "--ixprs",
        "-x",
        type=str,
        default="results/ixp_rs_asns.json",
        help="Path to the JSON file containing IXP RS ASNs",
    )
    parser.add_argument(
        "--output",
        "-o",
        type=str,
        default="results/",
        help="Directory to save the output JSON file",
    )
    args = parser.parse_args()
    setup_logging(args.debug)
    return args


def load_aspaths(filename: str) -> dict[str, dict[str, Any]]:
    with open(filename, "r") as f:
        data = json.loads(f.read())
    logging.debug(f"Loaded {len(data['paths'])} AS paths")
    return data["paths"]


def load_irr_asns(filename: str) -> dict[int, list[int]]:
    with open(filename, "r") as f:
        data: dict[str, list[int]] = json.loads(f.read())
    logging.debug(f"Loaded IRR ASNs for {len(data)} ASNs")
    return {int(asn): members for asn, members in data.items()}


def load_ixp_rs_asns(filename: str) -> list[int]:
    with open(filename, "r") as f:
        data = json.loads(f.read())
    logging.debug(f"Loaded {len(data)} IXP RS ASNs")
    return data


def check_communities(route: dict[str, Any], asns: list[int]) -> list[int]:
    community_asns: list[int] = []
    communities: list[list[int]] = route["communities"]

    for asn in asns:
        for community in communities:
            if community[0] == asn or community[1] == asn:
                community_asns.append(asn)
                logging.debug(
                    f"Community {community} indicates route {route['prefix']} might be via AS{asn}"
                )
                break

    if not community_asns:
        logging.debug(
            f"No communities indicate route {route['prefix']} is via ASNs {asns}"
        )
    return community_asns


def check_irr_for_asn(
    irr_asns: dict[int, list[int]], current_asn: int, next_asn: int
) -> list[int]:
    peer_asns: list[int] = []
    for asn in irr_asns[current_asn]:
        if asn not in irr_asns:
            logging.debug(
                f"AS{asn} does not have an AS-SET in IRR data, skipping"
            )
            continue
        if next_asn in irr_asns[asn]:
            logging.debug(f"AS{next_asn} found in AS-SET of AS{asn}")
            peer_asns.append(asn)
    logging.debug(
        f"AS{next_asn} is missing from AS-SET of AS{current_asn} and its members"
    )
    return peer_asns


def find_missing_asns(
    aspaths: dict[str, dict[str, Any]],
    irr_asns: dict[int, list[int]],
    ixp_rs_asns: list[int],
) -> dict[int, Any]:
    candidate_paths: dict[int, dict[str, Any]] = {}

    for as_path, route in aspaths.items():
        logging.debug(f"Processing AS path: {as_path}")

        asns = [int(a) for a in as_path.split(",")]
        for i in range(len(asns) - 1):
            current_asn = asns[i]
            next_asn = asns[i + 1]

            if current_asn in TIER1_ASNS:
                logging.debug(f"Skipping AS{current_asn} in Tier 1 AS list")
                continue

            if current_asn in ixp_rs_asns:
                logging.debug(f"Skipping AS{current_asn} in IXP RS list")
                continue

            if next_asn in irr_asns[current_asn]:
                logging.debug(
                    f"AS{next_asn} present in AS-SET of AS{current_asn}"
                )
                continue

            if i + 2 == len(asns):
                continue

            via_peer_asns = check_irr_for_asn(irr_asns, current_asn, next_asn)
            if not via_peer_asns:
                logging.debug(
                    f"AS{next_asn} is missing from AS-SET of AS{current_asn} and its peers"
                )
                continue

            community_asns = check_communities(route, via_peer_asns)
            if community_asns:
                logging.info(
                    f"Adding candidate path: {as_path}. {current_asn} -> {next_asn} could be via {community_asns}"
                )
                route["current_asn"] = current_asn
                route["next_asn"] = next_asn
                route["via_peer_asns"] = via_peer_asns
                route["community_asns"] = community_asns
                if current_asn not in candidate_paths:
                    candidate_paths[current_asn] = {}
                candidate_paths[current_asn][as_path] = route

    logging.info(
        f"Found {sum(len(paths) for paths in candidate_paths.values())} candidate paths with missing ASNs"
    )
    return candidate_paths


def write_json(data: Any, filename: str) -> None:
    with open(filename, "w") as f:
        json.dump(data, f, indent=2)
    logging.info(f"Wrote output to {filename}")


def main() -> None:
    args = parse_args()
    as_path = load_aspaths(args.aspaths)
    irr_asns = load_irr_asns(args.irr)
    ixp_rs_asns = load_ixp_rs_asns(args.ixprs)
    candidate_paths = find_missing_asns(as_path, irr_asns, ixp_rs_asns)
    write_json(candidate_paths, os.path.join(args.output, "missing_asns.json"))


if __name__ == "__main__":
    main()
