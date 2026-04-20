#!/usr/bin/env -S uv run --script
#
# /// script
# requires-python = "==3.13"
# dependencies = [
# "requests==2.32.5",
# ]
# ///

import argparse
import json
import logging
from typing import Any, Optional
import requests
import sqlite3
from globals import TIER1_ASNS, TIER1_ASSETS


def build_exclusions(assets: list[str]) -> str:
    return "\"" + "\", \"".join(assets) + "\""


def get_asn_asset(asn: int, db_cursor: sqlite3.Cursor) -> list[str]:
    logging.debug(f"Going to get data for ASN {asn}")

    db_cursor.execute(
        f"SELECT irr_as_set FROM peeringdb_network WHERE asn = {asn}"
    )
    row = db_cursor.fetchone()

    if not row:
        logging.error(f"No data found for ASN {asn}")
        return [""]
    asset: str = row[0]

    if not asset:
        logging.error(f"No AS-SET found for ASN {asn}")
        return [""]

    if len(asset.split()) == 1:
        # logging.info(f"AS-SET for ASN {asn} is {asset}")
        return [asset.split("::")[-1]]

    parts = asset.split()
    logging.warning(f"AS-SET for ASN {asn} contains multiple parts: {asset}")
    return [part.split("::")[-1] for part in parts]


def get_asn_assets(asns: list[int], db_path: str) -> dict[int, list[str]]:
    assets: dict[int, list[str]] = {}
    db_conn: sqlite3.Connection = sqlite3.connect(db_path)
    db_cursor: sqlite3.Cursor = db_conn.cursor()
    for asn in asns:
        if asn in TIER1_ASNS:
            logging.warning(f"Skipping Tier 1 AS{asn}")
            continue
        assets[asn] = get_asn_asset(asn, db_cursor)
    db_conn.close()
    return assets


def get_asset_asns(
    asset: str,
    asset_exclusions: str,
    url: str,
    session: Optional[requests.Session] = None,
) -> list[int]:
    logging.debug(f"Getting ASNs for AS-SET {asset} from IRRd server at {url}")

    rpsl_key = asset.split("::")[-1]
    if ":" in rpsl_key:
        rpsl_key = rpsl_key.split(":")[-1]

    if rpsl_key.upper().startswith("RS-"):
        logging.warning(f"Set {asset} looks like a route set, skipping")
        return []

    query = f"""
    query {{
        recursiveSetMembers(setNames: ["{rpsl_key}"], excludeSets: [{asset_exclusions}]) {{
            rpslPk, members
        }}
    }}
    """

    if session:
        response = session.post(f"{url}/graphql/", json={"query": query})
    else:
        response = requests.post(f"{url}/graphql/", json={"query": query})

    response.raise_for_status()
    assert response.status_code == 200

    data = json.loads(response.content)
    try:
        members = [
            int(member.lstrip("AS"))
            for irr_response in data["data"]["recursiveSetMembers"]
            for member in irr_response["members"]
        ]
    except Exception as e:
        logging.error(f"Error parsing response for AS-SET {asset}")
        raise e

    logging.info(f"Found {len(members)} ASNs in AS-SET {asset}")
    return members


def get_asns_for_assets(
    asn_assets: dict[int, list[str]], asset_exclusions: str, url: str
) -> dict[int, list[int]]:
    member_asns: dict[int, list[int]] = {}
    session = requests.Session()

    for asn, assets in asn_assets.items():
        unique_asns: set[int] = set()
        for asset in assets:
            if not asset:
                logging.warning(f"Skipping missing AS-SET for ASN {asn}")
                continue
            unique_asns.update(
                get_asset_asns(asset, asset_exclusions, url, session)
            )
        member_asns[asn] = list(unique_asns)
    return member_asns


def load_aspath_asns(input_file: str) -> list[int]:
    with open(input_file, "r") as f:
        data: dict[str, dict[str, Any]] = json.load(f)

    asns: set[int] = set()

    for as_path in data["paths"].keys():
        asns.update([int(asn) for asn in as_path.split(",")])
    logging.info(f"Extracted {len(asns)} unique ASNs from AS paths")
    return list(asns)


def load_divergent_asns(input_file: str) -> list[int]:
    with open(input_file, "r") as f:
        data: list[list[int]] = json.load(f)

    logging.info(f"Loaded {len(data)} ASNs")
    return [asn for (asn, _) in data]


def load_ixp_assets(input_file: str) -> list[str]:
    with open(input_file, "r") as f:
        data: list[str] = json.load(f)

    logging.info(f"Loaded {len(data)} IXP AS-SETs")
    return data


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Get all the ASNs that are in the IRR AS-SET for each ASN in the input list",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    parser.add_argument(
        "--db",
        "-b",
        type=str,
        help="Path to the SQLite database file containing PeeringDB data",
        default="peeringdb/peeringdb.sqlite3",
    )
    parser.add_argument(
        "--debug",
        "-d",
        action="store_true",
        help="Enable debug logging",
        default=False,
    )
    parser.add_argument(
        "--input",
        "-i",
        type=str,
        help="Path to the input JSON file containing ASNs",
        required=True,
    )
    parser.add_argument(
        "--ixp",
        type=str,
        help="Path to the input JSON file containing IXP AS-SETs",
        default="results/ixp_rs_assets.json",
    )
    parser.add_argument(
        "--output",
        "-o",
        type=str,
        help="Output JSON file to write the results to",
        default="results/irr_asns.json",
    )
    parser.add_argument(
        "--url",
        type=str,
        help="Base URL to the IRRd server to query for AS-SET membership",
        default="https://irrd.as5405.net/",
    )
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument(
        "--divergent",
        action="store_true",
        help="Input file list is list of divergent ASNs, get ASNs for only those ASes",
    )
    group.add_argument(
        "--aspaths",
        action="store_true",
        help="Input file list is list of AS paths, get ASNs for all ASes in those paths",
    )
    setup_logging(parser.parse_args().debug)
    return parser.parse_args()


def setup_logging(debug: bool) -> None:
    level = logging.DEBUG if debug else logging.INFO

    logging.basicConfig(
        format="%(asctime)s|%(levelname)s|%(process)d|%(funcName)s|%(message)s",
        level=level,
        handlers=[
            logging.StreamHandler(),
        ],
    )


def write_json(filename: str, data: dict[Any, Any]) -> None:
    with open(filename, "w") as f:
        f.write(json.dumps(data, indent=2))
    logging.info(f"Wrote data to {filename}")


def main():
    args = parse_args()

    asns = []
    if args.divergent:
        logging.info("Input file is list of divergent ASNs")
        asns = load_divergent_asns(args.input)
    elif args.aspaths:
        logging.info("Input file is list of AS paths")
        asns = load_aspath_asns(args.input)

    assets = get_asn_assets(asns, args.db)
    ixp_assets = load_ixp_assets(args.ixp)
    asset_exclusions = build_exclusions(ixp_assets + TIER1_ASSETS)
    irr_asns = get_asns_for_assets(assets, asset_exclusions, args.url)
    write_json(args.output, irr_asns)


if __name__ == "__main__":
    main()
