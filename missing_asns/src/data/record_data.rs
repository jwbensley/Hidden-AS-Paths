use crate::data::paths::Paths;
use bgpkit_parser::MrtRecord;
use std::sync::{Arc, RwLock};
/// Shared data that is passed around when parsing an individual MRT entry/record
pub struct RecordData<'a> {
    pub(crate) mrt_record: &'a MrtRecord,
    pub(crate) paths: &'a Arc<RwLock<Paths>>,
    pub(crate) mrt_fp: &'a String,
}

impl<'a> RecordData<'a> {
    pub fn new(
        mrt_record: &'a MrtRecord,
        paths: &'a Arc<RwLock<Paths>>,
        mrt_fp: &'a String,
    ) -> Self {
        Self {
            mrt_record,
            paths,
            mrt_fp,
        }
    }
}
