#!/usr/bin/env -S uv run --script
#
# /// script
# requires-python = ">=3.12"
# dependencies = [
# "lz4==4.4.5",
# "orjson==3.11.8",
# "requests==2.32.5",
# ]
# ///

import argparse
import csv
import gzip
import logging
import lz4.frame
import orjson
import os
import requests
import tempfile
from collections import defaultdict
from typing import Any


def download_files(timestamp: str, output_path: str) -> list[str]:
    yyyy = timestamp[:4]
    mm = timestamp[4:6]
    dd = timestamp[6:8]

    downloaded_files: list[str] = []

    for family in ['ipv4', 'ipv6']:
        filename = f"ihr_hegemony_{family}_local_{yyyy}-{mm}-{dd}.csv.lz4"
        url = f"https://archive.ihr.live/ihr/hegemony/{family}/local/{yyyy}/{mm}/{dd}/{filename}"
        output_file = tempfile.mkstemp()[1]

        logging.info(f"Downloading file from {url} to {output_file}")
        response = requests.get(url)
        response.raise_for_status()
        with open(output_file, "wb") as f:
            f.write(response.content)
        downloaded_files.append(output_file)
        logging.info(f"File downloaded")

    return downloaded_files


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Load the AS Hegemony data for a specific day, and merge one day's worth of data into a single JSON file. "
        "The JSON contains all ASNs that are related to the same origin ASN. "
        "Specify a local file to parse or a date to download the data for."
    )
    parser.add_argument(
        "--debug",
        "-d",
        action="store_true",
        help="Enable debug logging",
        default=False,
    )
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument(
        "--timestamp",
        "-t",
        type=str,
        help="Date in YYYYMMDD format",
    )
    group.add_argument(
        "--input",
        "-i",
        type=str,
        help="Path to an existing Hegemony file.",
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


def parse_csv(filenames: list[str]) -> dict[Any, Any]:
    hegemony_data: dict[str, dict[str, float]] = defaultdict(dict)

    for filename in filenames:
        logging.info(f"Parsing file {filename}")
        with lz4.frame.open(filename, mode="rt", encoding="utf-8") as csvfile:  # type: ignore
            reader = csv.DictReader(csvfile)  # type: ignore
            for row in reader:
                originasn = row["originasn"]
                asn = row["asn"]
                hege = float(row["hege"])
                hegemony_data[originasn][asn] = hege

    logging.info(f"Parsed {len(hegemony_data)} origin ASNs with hegemony data")
    return dict(hegemony_data)


def setup_logging(debug: bool) -> None:
    level = logging.DEBUG if debug else logging.INFO

    logging.basicConfig(
        format="%(asctime)s|%(levelname)s|%(process)d|%(funcName)s|%(message)s",
        level=level,
        handlers=[
            logging.StreamHandler(),
        ],
    )


def write_to_json(
    data: dict[Any, Any],
    output_filename: str,
    compress: bool = False,
) -> None:
    if compress:
        with gzip.open(output_filename, 'wt', encoding='utf-8') as jsonfile:
            jsonfile.write(
                orjson.dumps(data, option=orjson.OPT_INDENT_2).decode('utf-8')
            )
    else:
        with open(output_filename, 'w', encoding='utf-8') as jsonfile:
            jsonfile.write(
                orjson.dumps(data, option=orjson.OPT_INDENT_2).decode('utf-8')
            )
    logging.info(f"Data written to {output_filename}")


def main() -> None:
    args = parse_args()

    if args.timestamp:
        input_files = download_files(args.timestamp, args.output)
    else:
        input_files = [args.input]

    hegemony_data = parse_csv(input_files)

    output_filename = os.path.join(
        args.output,
        f"ihr_hegemony_local.json",
    )
    write_to_json(hegemony_data, output_filename, False)


if __name__ == "__main__":
    main()
