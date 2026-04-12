#!/usr/bin/env -S uv run --script
#
# /// script
# requires-python = ">=3.12"
# dependencies = [
# "requests==2.32.5",
# ]
# ///

import argparse
import json
import logging
from typing import Any
import requests


def get_asset_asns(asset: str, url: str) -> list[int]:
    logging.debug(f"Getting ASNs for AS-SET {asset} from IRRd server at {url}")

    query = f"""
    query {{
        recursiveSetMembers(setNames: ["{asset}"]) {{
            rpslPk, members
        }}
    }}
    """

    response = requests.post(f"{url}/graphql/", json={"query": query})
    response.raise_for_status()
    assert response.status_code == 200

    data = json.loads(response.content)
    members = [
        int(member.lstrip("AS"))
        for irr_response in data["data"]["recursiveSetMembers"]
        for member in irr_response["members"]
    ]

    logging.info(f"Found {len(members)} ASNs in AS-SET {asset}")
    return members


def get_asset_asns_for_assets(
    asn_assets: dict[int, list[str]], url: str
) -> dict[int, list[int]]:
    member_asns: dict[int, list[int]] = {}
    for asn, assets in asn_assets.items():
        unique_asns: set[int] = set()
        for asset in assets:
            if not asset:
                logging.warning(f"Skipping missing AS-SET for ASN {asn}")
                continue
            unique_asns.update(get_asset_asns(asset, url))
        member_asns[asn] = list(unique_asns)
    return member_asns


def load_asn_assets(input_file: str) -> dict[int, list[str]]:
    with open(input_file, "r") as f:
        data: dict[int, list[str]] = json.load(f)

    logging.info(f"Loaded AS-SETs for {len(data)} ASNs")
    return data


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Get all the ASNs that are in the IRR AS-SET for each ASN in the input list"
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
        help="Input JSON file containing list of ASNs and their AS-SETs",
        default="results/asn_assets.json",
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
    asn_assets = load_asn_assets(args.input)
    irr_asns = get_asset_asns_for_assets(asn_assets, args.url)
    write_json(args.output, irr_asns)


if __name__ == "__main__":
    main()
