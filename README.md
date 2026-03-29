# Hidden AS Hops

Searching for BGP AS Paths which indicated that one or more ASNs have been removed from the AS path.

## Running

Ensure we have a local copy of PeeringDB:

```shell
cd peeringdb/
python3 -m venv .venv
source .venv/bin/activate
python3 -m pip install --upgrade pip
python3 -m pip install peeringdb django_peeringdb

peeringdb sync

# Test
$ sqlite3 -json peeringdb.sqlite3 "select asn from peeringdb_network where info_type = 'Route Server' or info_types like '%Route Server%' order by asn desc;" | jq 'map(.[]) | length'
750
```

```shell
# Install requirements
sudo apt install libsqlite3-dev

# Build
cargo build -r

# Download MRTs
./target/release/hidden-as-paths download -y 2026-02-04

# Parse MRTs and filter results
./target/release/hidden-as-paths file -f /opt/mrts/20260204/ris.rrc18.bview.20260204.0000.gz
```

## Testing

```shell
cargo build
export RUST_BACKTRACE=full; cargo test -- --nocapture
```
