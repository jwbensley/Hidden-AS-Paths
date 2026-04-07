#!/usr/bin/env -S uv run --script
#
# /// script
# requires-python = ">=3.12"
# dependencies = [
# "lz4==4.4.5",
# "orjson==3.11.8",
# ]
# ///

import csv
from collections import defaultdict
import sys
import orjson
import gzip
import lz4.frame


def parse_csv(filename: str) -> dict[str, dict[str, float]]:
    hegemony_data: dict[str, dict[str, float]] = defaultdict(dict)

    with lz4.frame.open(filename, mode='rt', encoding='utf-8') as csvfile:
        reader = csv.DictReader(csvfile)
        for row in reader:
            originasn = row['originasn']
            asn = row['asn']
            hege = float(row['hege'])
            hegemony_data[originasn][asn] = hege

        return dict(hegemony_data)


def write_to_json(
    data: dict[str, dict[str, float]],
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
    print(f"Data written to {output_filename}")


if __name__ == "__main__":
    filename = sys.argv[1]
    hegemony_dict = parse_csv(filename)
    write_to_json(hegemony_dict, filename.replace('.csv.lz4', '.json'), False)
