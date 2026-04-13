#!/usr/bin/env -S uv run --script
#
# /// script
# requires-python = ">=3.12"
# dependencies = [
# "orjson==3.11.8"
# ]
# ///

import argparse
import logging
from typing import Any
import orjson


def load_divergent_asns(filename: str) -> dict[int, int]:
    with open(filename, "r") as f:
        data: list[tuple[int, int]] = orjson.loads(f.read())
    return {int(asn): int(count) for (asn, count) in data}


def is_private(asn: int) -> bool:
    if asn == 0:
        # RFC 7607
        return True
    elif asn == 23456:
        # RFC 4893
        return True
    elif 64496 <= asn <= 64511:
        # RFC 5398
        return True
    elif 64512 <= asn <= 65535:
        # RFC 6996
        return True
    elif 65536 <= asn <= 65551:
        # RFC 5398
        return True
    elif 65552 <= asn <= 131071:
        # IANA reserved
        return True
    elif 4200000000 <= asn <= 4294967295:
        # RFC 6996
        return True
    else:
        return False


def weight_paths(filename: str, divergent_asns: dict[int, int]) -> None:
    paths: dict[Any, Any] = orjson.loads(open(filename, "r").read())

    origin_as_paths: dict[str, list[dict[str, Any]]]
    for origin_as_paths in paths["paths"].values():
        for as_path in origin_as_paths["as_paths"]:
            asns: list[int] = as_path["as_path"]
            routes: list[dict[str, Any]] = as_path["routes"]

            asn_weight = 0
            pos = -1
            for asn in asns:
                if asn in divergent_asns:
                    pos = asns.index(asn)
                    break

            if pos != -1:
                next_asn = asns[pos + 1]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Scan divergent AS paths and assign a weight to them based on various factors"
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


def main() -> None:
    args = parse_args()
    divergent_asns = load_divergent_asns(args.asns)
    weight_paths(args.paths, divergent_asns)


if __name__ == "__main__":
    main()
