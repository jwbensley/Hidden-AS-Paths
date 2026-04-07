#!/usr/bin/env -S uv run --script
#
# /// script
# requires-python = ">=3.12"
# dependencies = [
# "orjson==3.11.8",
# ]
# ///

from typing import Any

import orjson
import sys


def get_as_path_count_per_origin(filename: str) -> dict[str, int]:
    with open(filename, "r") as f:
        data: dict[str, dict[str, dict[str, Any]]] = orjson.loads(f.read())

    counts: dict[str, int] = {}

    for origin_asn, data in data["paths"].items():
        counts[origin_asn] = len(data["as_paths"])

    sorted_counts = dict(
        sorted(counts.items(), key=lambda item: item[1], reverse=True)
    )
    print(f"Count of AS paths per origin ASN:")
    print(orjson.dumps(sorted_counts, option=orjson.OPT_INDENT_2).decode())
    return sorted_counts


def get_freq_of_peer_asn(filename: str) -> dict[str, int]:
    with open(filename, "r") as f:
        data: dict[str, dict[str, dict[str, list[dict[str, int]]]]] = (
            orjson.loads(f.read())
        )

    counts: dict[str, int] = {}

    for data in data["paths"].values():
        for as_path in data["as_paths"]:
            peer_asn = str(as_path["as_path"][0])
            if peer_asn not in counts:
                counts[peer_asn] = 0
            counts[peer_asn] += 1

    sorted_counts = dict(
        sorted(counts.items(), key=lambda item: item[1], reverse=True)
    )
    print(f"Frequency of peer ASN:")
    print(orjson.dumps(sorted_counts, option=orjson.OPT_INDENT_2).decode())
    return sorted_counts


def main():
    if len(sys.argv) != 2 or not sys.argv[1]:
        print(f"Usage: {sys.argv[0]} <filename>")
        sys.exit(1)
    filename = sys.argv[1]
    get_as_path_count_per_origin(filename)
    get_freq_of_peer_asn(filename)


if __name__ == "__main__":
    main()
