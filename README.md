# Hidden AS Hops

Searching for BGP AS Paths which indicated that one or more ASNs have been removed from the AS path.

## Setup

Ensure we have a local copy of PeeringDB:

```shell
cd peeringdb/

# Set your PeeringDB API key in config.yaml
python3 -m venv .venv
source .venv/bin/activate
python3 -m pip install --upgrade pip
python3 -m pip install peeringdb django_peeringdb

peeringdb sync

cd ../
```

Install uv and python:

```shell
python3 -m pip install uv
uv python install 3.13
uv python install 3.14
```

Install rust dependencies:

```shell
sudo apt install libsqlite3-dev
```

## Missing ASNs

```shell
cd missing_asns/

cargo build
export RUST_BACKTRACE=full; cargo test -- --nocapture

cargo build -r
cd ../

# Yesterday's yyyy-mm-dd
YMD=$(date "+%Y-%m-%d" --date="yesterday")

# Download MRTs
./divergent_paths/target/release/hidden-as-paths -t 10 download -y $YMD -p /opt/mrts/$YMD

# Parse MRTs and filter results
./divergent_paths/target/release/hidden-as-paths -t 10 files -f /opt/mrts/$YMD/*

./python/get_irr_asns.py --input results/as_paths.json --divergent
./python/find_missing_asns.py
```

## Divergent AS Paths

```shell
cd divergent_paths/

cargo build
export RUST_BACKTRACE=full; cargo test -- --nocapture

cargo build -r
cd ../

# Yesterday's yyyy-mm-dd
YMD=$(date "+%Y-%m-%d" --date="yesterday")

# Download MRTs
./divergent_paths/target/release/hidden-as-paths -t 10 download -y $YMD -p /opt/mrts/$YMD

# Parse MRTs and filter results
./divergent_paths/target/release/hidden-as-paths -t 10 files -f /opt/mrts/$YMD/*

# Pull weighting data
./python/get_hegemony.py --timestamp $YMD
./python/get_ixp_data.py
./python/get_irr_asns.py --input results/diverging_asn_count.json --divergent

# Weight paths
./python/weight_paths.py
```
