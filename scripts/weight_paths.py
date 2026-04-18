#!/usr/bin/env -S uv run --script
#
# /// script
# requires-python = "==3.13"
# ///

import argparse
import gzip
import logging
import json
import os
from typing import Any


def load_divergent_asns(filename: str) -> dict[int, int]:
    with open(filename, "r") as f:
        data: list[tuple[int, int]] = json.loads(f.read())
    logging.debug(f"Loaded divergent ASN counts for {len(data)} ASNs")
    return {int(asn): int(count) for (asn, count) in data}


def load_hegemony(filename: str) -> dict[int, dict[int, float]]:
    with open(filename, "r") as f:
        data: dict[str, dict[str, float]] = json.loads(f.read())
    logging.debug(f"Loaded hegemony data for {len(data)} ASNs")
    return {
        int(asn): {
            int(member_asn): float(score)
            for member_asn, score in asn_data.items()
        }
        for asn, asn_data in data.items()
    }


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


def check_route_communities(
    communities: list[list[int]], expected_community_asns: set[int]
) -> tuple[int, list[list[int]]]:

    weight = 0

    for community in communities[:]:
        community_asn, community_value = community[0], community[1]

        # Remove communities with expected ASNs
        if community_asn in expected_community_asns:
            communities.remove(community)
            continue

        # For communities with an ASN that isn't in the expected set of ASNs for this path...

        # If it's not a BOGON ASN, assign a weight to this community list
        if not is_bogon_asn(community_asn):
            weight = 10
        else:
            # If it is a BOGON ASN, check the value part of the community
            assert (
                community_value >= 0
            ), f"Community value should be non-negative: {community}"

    return weight, communities


def is_bogon_asn(asn: int) -> bool:
    if asn == 0:
        # RFC 7607
        return True
    elif asn == 23456:
        # RFC 4893
        return True
    elif asn >= 64496 and asn <= 64511:
        # RFC 5398
        return True
    elif asn >= 64512 and asn <= 65535:
        # RFC 6996
        return True
    elif asn >= 65536 and asn <= 65551:
        # RFC 5398
        return True
    elif asn >= 65552 and asn <= 131071:
        # IANA reserved
        return True
    elif asn >= 4200000000 and asn <= 4294967295:
        # RFC 6996
        return True
    else:
        return False


def weight_paths(
    filename: str,
    divergent_asns: dict[int, int],
    hegemony: dict[int, dict[int, float]],
    irr_asns: dict[int, list[int]],
    ixp_rs_asns: list[int],
) -> dict[int, Any]:

    logging.info(f"Loading paths from {filename}")
    results: dict[int, list[dict[str, Any]]] = {}
    paths: dict[str, dict[str, Any]] = dict(
        json.loads(open(filename, "r").read())
    )

    origin_as_paths: dict[str, list[Any]]
    for origin_as, origin_as_paths in paths["paths"].items():
        logging.debug(f"Checking paths for origin AS {origin_as}")
        path_diverging_asns: list[int] = origin_as_paths["diverging_asns"]

        as_path: dict[str, Any]
        for as_path in origin_as_paths["as_paths"]:
            asns: list[int] = as_path["as_path"]
            expected_community_asns = set(asns + ixp_rs_asns + [0])
            routes: list[dict[str, Any]] = as_path["routes"]

            if len(asns) < 3:
                continue

            divergent_asn = -1
            next_asn = -1
            for asn in asns:
                if asn in path_diverging_asns:
                    if asns.index(asn) + 1 > len(asns) - 1:
                        continue
                    divergent_asn = asn
                    next_asn = asns[asns.index(asn) + 1]

            assert divergent_asn > 0
            assert next_asn > 0

            # Check irr membership
            if divergent_asn not in irr_asns:
                logging.debug(
                    f"Divergent ASN {divergent_asn} not found in IRR data"
                )
                irr_weight = 0
            else:
                if next_asn not in irr_asns[divergent_asn]:
                    irr_weight = 10
                else:
                    irr_weight = 0

            if irr_weight == 0:
                continue

            # Check hegemony
            if divergent_asn not in hegemony:
                logging.debug(
                    f"Divergent ASN {divergent_asn} not found in hegemony data"
                )
                hegemony_weight = 0
            else:
                if next_asn not in hegemony[divergent_asn]:
                    hegemony_weight = 10
                else:
                    hegemony_weight = 0

            # Find any route on this path that has a community that doesn't match the expected set of ASNs
            community_weight = 0
            suspect_route = {}
            for route in routes:
                communities: list[list[int]] = route["communities"]
                community_weight, stripped_communities = (
                    check_route_communities(
                        communities, expected_community_asns
                    )
                )
                if community_weight > 0:
                    suspect_route = route
                    suspect_route["communities"] = stripped_communities
                break

            if not suspect_route:
                continue

            if not divergent_asn in results:
                results[divergent_asn] = []
            results[divergent_asn].append(
                {
                    "as_path": asns,
                    "divergent_asn": divergent_asn,
                    "divergent_asn_score": divergent_asns[divergent_asn],
                    "suspect_route": suspect_route,
                    "community_weight": community_weight,
                    "hegemony_weight": hegemony_weight,
                    "irr_weight": irr_weight,
                    "score": community_weight + hegemony_weight + irr_weight,
                }
            )

    return results


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Scan divergent AS paths and assign a weight to them based on various factors",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    parser.add_argument(
        "--asns",
        "-a",
        type=str,
        default="results/diverging_asn_count.json",
        help="Path to the JSON file containing diverging ASNs and their counts",
    )
    parser.add_argument(
        "--debug",
        "-d",
        action="store_true",
        help="Enable debug logging",
        default=False,
    )
    parser.add_argument(
        "--hegemony",
        "-e",
        type=str,
        default="results/ihr_hegemony_local.json",
        help="Path to the JSON file containing AS hegemony data",
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
        "--paths",
        "-p",
        type=str,
        default="results/divergent_paths.json",
        help="Path to the AS paths file to be weighted",
    )
    parser.add_argument(
        "--output",
        "-o",
        type=str,
        default="results/",
        help="Path to output directory where results will be written",
    )
    args = parser.parse_args()
    setup_logging(args.debug)
    return args


def setup_logging(debug: bool) -> None:
    level = logging.DEBUG if debug else logging.INFO

    logging.basicConfig(
        format="%(asctime)s|%(levelname)s|%(process)d|%(funcName)s|%(message)s",
        level=level,
        handlers=[
            logging.StreamHandler(),
        ],
    )


def suspect_route_count(results: dict[int, Any]) -> list[list[int]]:
    # Return list sorted by count of suspect routes for each divergent ASN
    return sorted(
        [
            [asn, len(suspect_routes)]
            for asn, suspect_routes in results.items()
        ],
        key=lambda x: x[1],
        reverse=True,
    )


def write_to_json(
    data: Any,
    output_filename: str,
    compress: bool = False,
) -> None:
    if compress:
        with gzip.open(output_filename, 'wt', encoding='utf-8') as jsonfile:
            jsonfile.write(json.dumps(data, indent=2))
    else:
        with open(output_filename, 'w', encoding='utf-8') as jsonfile:
            jsonfile.write(json.dumps(data, indent=2))
    logging.info(f"Wrote to {output_filename}")


def main() -> None:
    args = parse_args()
    divergent_asns = load_divergent_asns(args.asns)
    hegemony = load_hegemony(args.hegemony)
    irr_asns = load_irr_asns(args.irr)
    ixp_rs_asns = load_ixp_rs_asns(args.ixprs)
    results = weight_paths(
        args.paths, divergent_asns, hegemony, irr_asns, ixp_rs_asns
    )
    write_to_json(results, os.path.join(args.output, "weighted_paths.json"))
    count_by_asn = suspect_route_count(results)
    write_to_json(
        count_by_asn,
        os.path.join(args.output, "suspect_route_count.json"),
    )


if __name__ == "__main__":
    main()
