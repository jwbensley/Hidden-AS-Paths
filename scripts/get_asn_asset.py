#!/usr/bin/env -S uv run --script
#
# /// script
# requires-python = ">=3.12"
# ///


import argparse
import json
import logging
from typing import Any
import sqlite3


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
        assets[asn] = get_asn_asset(asn, db_cursor)
    db_conn.close()
    return assets


def load_asns(input_file: str) -> list[int]:
    with open(input_file, "r") as f:
        data: list[list[int]] = json.load(f)

    logging.info(f"Loaded {len(data)} ASNs")
    return [asn for (asn, _) in data]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Get the AS-SET for each ASN in the input JSON file"
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
        default="results/diverging_asn_count.json",
    )
    parser.add_argument(
        "--db",
        type=str,
        help="Path to the SQLite database file containing PeeringDB data",
        default="peeringdb/peeringdb.sqlite3",
    )
    parser.add_argument(
        "--output",
        "-o",
        type=str,
        help="Path to the output JSON file to write AS-SETs",
        default="results/asn_assets.json",
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
    asns = load_asns(args.input)
    assets = get_asn_assets(asns, args.db)
    write_json(args.output, assets)


if __name__ == "__main__":
    main()
