use crate::types::asn::Asn;
use log::{debug, info};
use rusqlite::Connection;

pub fn get_ixp_rs_asns(filename: &String) -> Vec<Asn> {
    debug!("Loading peeringdb data from {:?}", filename);
    let conn = Connection::open(filename).unwrap();

    let mut stmt = conn
        .prepare(
            "SELECT asn FROM peeringdb_network WHERE info_type = 'Route Server' ORDER BY asn DESC",
        )
        .unwrap();

    let asns: Vec<Asn> = stmt
        .query_map([], |row| row.get(0))
        .unwrap()
        .map(|res| Asn::new(res.unwrap()))
        .collect();

    info!("Loaded {} ASNs from PeeringDB", asns.len());
    asns
}
