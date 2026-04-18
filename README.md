# Hidden AS Hops

Searching for BGP AS Paths which indicated that one or more ASNs have been removed from the AS path.

## Setup

Ensure we have a local copy of PeeringDB:

```shell
cd peeringdb/

# Set your PeeringDB API key in config.yaml !!

python3 -m venv .venv
source .venv/bin/activate
python3 -m pip install --upgrade pip
python3 -m pip install peeringdb django_peeringdb

peeringdb sync
```

Install uv and python:

```shell
python3 -m pip install uv
uv python install 3.13
```

Install rust dependencies:

```shell
sudo apt install libsqlite3-dev
```

## Running

```shell
# Build
cargo build -r

# Yesterday's yyyy-mm-dd
YMD=$(date "+%Y-%m-%d" --date="yesterday")

# Download MRTs
./target/release/hidden-as-paths -t 10 download -y $YMD -p /opt/mrts/$YMD

# Parse MRTs and filter results
./target/release/hidden-as-paths -t 10 file -f /opt/mrts/$YMD/ris.rrc18*

# Pull weighting data
./scripts/get_hegemony.py --timestamp $YMD
./scripts/get_ixprs_asns.py
./scripts/get_irr_asns.py

# Weight paths
./scripts/weight_paths.py
```

## Testing

```shell
cargo build
export RUST_BACKTRACE=full; cargo test -- --nocapture
```
