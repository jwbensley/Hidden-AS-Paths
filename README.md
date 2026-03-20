# Hidden AS Hops

Searching for BGP AS Paths with stripped ASNs...

```shell
sudo apt install libsqlite3-dev
cargo build
export RUST_BACKTRACE=full; cargo test -- --nocapture

cargo build -r
./target/release/hidden-as-paths file -f /opt/mrts/20260204/ris.rrc18.bview.20260204.0000.gz
```

```shell
cd peeringdb/
python3 -m venv .venv
source .venv/bin/activate
python3 -m pip install --upgrade pip
python3 -m pip install peeringdb django_peeringdb
# Test
sqlite3 -json peeringdb.sqlite3 "select asn from peeringdb_network where info_type = 'Route Server' order by asn desc;" | jq 'map(.[])'
```
