use log::{debug, info};
use rusqlite::Connection;

pub fn get_ixp_rs_asns(filename: &String) -> Vec<u32> {
    debug!("Loading peeringdb data from {:?}", filename);
    let conn = Connection::open(filename).unwrap();

    let mut stmt = conn
        .prepare(
            "SELECT asn FROM peeringdb_network WHERE info_type = 'Route Server' ORDER BY asn DESC",
        )
        .unwrap();

    let mut asns: Vec<u32> = stmt
        .query_map([], |row| row.get(0))
        .unwrap()
        .map(|res| res.unwrap())
        .collect();

    // ASN 0 is a special ASN used by many networks as an action community.
    asns.insert(0, 0);

    info!("Loaded {} ASNs from PeeringDB", asns.len());
    asns
}
