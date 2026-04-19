use crate::types::asn::Asn;
use log::{debug, info};
use rusqlite::Connection;

pub fn get_ixp_rs_asns(db_filename: &String, json_filename: &String) -> Vec<Asn> {
    debug!("Loading peeringdb data from {:?}", db_filename);
    let conn = Connection::open(db_filename).unwrap();

    let mut stmt = conn
        .prepare(
            "SELECT asn FROM peeringdb_network WHERE info_type = 'Route Server' OR info_types LIKE '%Route Server%' ORDER BY asn DESC",
        )
        .unwrap();

    let asns: Vec<Asn> = stmt
        .query_map([], |row| row.get(0))
        .unwrap()
        .map(|res| Asn::new(res.unwrap()))
        .collect();
    info!("Loaded {} ASNs from PeeringDB", asns.len());

    serde_json::to_writer_pretty(std::fs::File::create(json_filename).unwrap(), &asns).unwrap();
    info!("Saved PeeringDB ASNs to {:?}", json_filename);

    asns
}
