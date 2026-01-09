# Hidden AS Hops

Searching for hidden ASNs in BGP AS Paths...

```shell
cargo build
cargo test -- --nocapture

cargo build -r
./target/release/hidden-as-paths file -f ./mrts/route-views.mwix.rib.20250922.0000.bz2 ./mrts/route-views.bknix.rib.20250922.0000.bz2
```
