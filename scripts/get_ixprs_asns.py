#!/usr/bin/env -S uv run --script
#
# /// script
# requires-python = ">=3.12"
# ///

import argparse
import json
import os
import sqlite3


def main():
    parser = argparse.ArgumentParser(
        description='Get all IXP RS ASNs from PeeringDB'
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
        default="results/ixp_rs_asns.json",
        help="Path to the output JSON file",
    )
    args = parser.parse_args()

    # If the database file does not exist, print an error message and exit
    # Otherwise sqliet3 will create an empty database file and the query will return no results
    if not os.path.exists(args.db):
        print(f"Error: Database file '{args.db}' does not exist.")
        return

    conn = sqlite3.connect(args.db)
    cursor = conn.cursor()
    query = """
    SELECT asn FROM peeringdb_network WHERE info_type = 'Route Server' OR info_types LIKE '%Route Server%' ORDER BY asn DESC;
    """
    cursor.execute(query)
    asns = cursor.fetchall()
    conn.close()

    with open(args.output, 'w') as f:
        f.write(json.dumps([asn[0] for asn in asns], indent=2))
    print(f"Saved {len(asns)} ASNs to {args.output}")


if __name__ == '__main__':
    main()
