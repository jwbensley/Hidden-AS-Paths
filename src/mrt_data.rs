use crate::mrt_types::peer::PeerTable;
use crate::data::paths::Paths;
use bgpkit_parser::MrtRecord;
use std::sync::{Arc, RwLock};
/// Shared data that is passed around when parsing an individual MRT entry/record
pub struct MrtData<'a> {
    pub(crate) mrt_record: &'a MrtRecord,
    pub(crate) paths: &'a Arc<RwLock<Paths>>,
    pub(crate) peer_table: &'a PeerTable,
    pub(crate) mrt_fp: &'a String,
}

impl<'a> MrtData<'a> {
    pub fn new(
        mrt_record: &'a MrtRecord,
        paths: &'a Arc<RwLock<Paths>>,
        peer_table: &'a PeerTable,
        mrt_fp: &'a String,
    ) -> Self {
        Self {
            mrt_record,
            paths,
            peer_table,
            mrt_fp,
        }
    }
}
