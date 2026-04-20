#!/usr/bin/env -S uv run --script
#
# /// script
# requires-python = "==3.14"
# ///

import argparse
import json
import os
import sqlite3
from typing import Any


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Get all IXP RS ASNs from PeeringDB",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    parser.add_argument(
        "--db",
        type=str,
        default="peeringdb/peeringdb.sqlite3",
        help="Path to the SQLite database",
    )
    parser.add_argument(
        "--output",
        type=str,
        default="results/",
        help="Path to the output directory",
    )
    args = parser.parse_args()
    return args


def query_db(db_path: str, query: str) -> Any:
    conn = sqlite3.connect(db_path)
    cursor = conn.cursor()
    cursor.execute(query)
    asns = cursor.fetchall()
    conn.close()
    return asns


def get_ixp_rs_asns(db_path: str) -> list[int]:
    query = "SELECT asn FROM peeringdb_network WHERE info_type = 'Route Server' OR info_types LIKE '%Route Server%' ORDER BY asn DESC;"
    results = query_db(db_path, query)
    return [asn[0] for asn in results]


def get_ixp_rs_assets(db_path: str) -> list[str]:
    query = "SELECT irr_as_set FROM peeringdb_network WHERE info_type = 'Route Server' OR info_types LIKE '%Route Server%' ORDER BY asn DESC;"
    results = query_db(db_path, query)
    assets: list[str] = []
    for result in results:
        asset = result[0]
        if asset:
            for a in asset.split():
                assets.append(a.split("::")[-1])
    return assets


def write_json(data: Any, filename: str) -> None:

    with open(filename, 'w') as f:
        f.write(json.dumps(data, indent=2))
    print(f"Saved to {filename}")


def main():
    args = parse_args()

    # If the database file does not exist, print an error message and exit
    # Otherwise sqlite3 will create an empty database file and the query will return no results
    if not os.path.exists(args.db):
        print(f"Error: Database file '{args.db}' does not exist.")
        return

    if not os.path.exists(args.output):
        os.makedirs(args.output)

    ixp_rs_asns = get_ixp_rs_asns(args.db)
    filename = os.path.join(args.output, "ixp_rs_asns.json")
    write_json(ixp_rs_asns, filename)

    ixp_rs_assets = get_ixp_rs_assets(args.db)
    filename = os.path.join(args.output, "ixp_rs_assets.json")
    write_json(ixp_rs_assets, filename)


if __name__ == '__main__':
    main()
