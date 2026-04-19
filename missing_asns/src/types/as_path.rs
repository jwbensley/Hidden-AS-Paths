use crate::types::asn::Asn;
use bgpkit_parser::models::Asn as BgpKitAsn;
use serde::{Serialize, Serializer};
use std::fmt;
use std::hash::Hash;

/// A deduped AS path which can be serialised to JSON.
#[derive(Debug, Clone, Eq, Default, Ord, PartialOrd)]
pub struct AsPath {
    as_path: Vec<Asn>,
}

impl fmt::Display for AsPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // convert the path into a stable string form, e.g. "64512 64513 64514"
        write!(f, "{}", self.to_string_representation())
    }
}

impl Serialize for AsPath {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialise the AS path as a string. The AS path is the key in the hashmap,
        // keys must be strings to be serialised to JSON.
        serializer.serialize_str(&self.to_string_representation())
    }
}

impl PartialEq for AsPath {
    fn eq(&self, other: &Self) -> bool {
        self.as_path == other.as_path
    }
}

impl Hash for AsPath {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_path.hash(state);
    }
}

impl AsPath {
    pub fn new(mut as_path: Vec<Asn>) -> Self {
        as_path.dedup();
        AsPath { as_path }
    }

    pub fn get_mock(as_path: Option<Vec<Asn>>) -> AsPath {
        let as_path = as_path.unwrap_or(Vec::from([
            Asn::get_mock(Some(3)),
            Asn::get_mock(Some(2)),
            Asn::get_mock(None),
        ]));
        AsPath::new(as_path)
    }

    pub fn get_asns(&self) -> &Vec<Asn> {
        &self.as_path
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        self.as_path.len()
    }

    pub fn from_vec(asns: &[BgpKitAsn]) -> Self {
        let as_path = asns
            .iter()
            .map(|a| Asn::new(a.to_u32()))
            .collect::<Vec<Asn>>();
        AsPath::new(as_path)
    }

    pub fn to_string_representation(&self) -> String {
        self.as_path
            .iter()
            .map(|asn| asn.clone().to_u32().to_string())
            .collect::<Vec<_>>()
            .join(",")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let as_path = vec![Asn::new(1), Asn::new(2), Asn::new(3)];
        let ap = AsPath::new(as_path.clone());

        assert_eq!(ap.len(), as_path.len());
        assert_eq!(ap.get_asns(), &as_path);
    }

    #[test]
    fn test_new_dedups_consecutive_asns() {
        let as_path = vec![
            Asn::new(1),
            Asn::new(1),
            Asn::new(2),
            Asn::new(2),
            Asn::new(3),
        ];
        let ap = AsPath::new(as_path);

        assert_eq!(ap.len(), 3);
        assert_eq!(ap.get_asns(), &vec![Asn::new(1), Asn::new(2), Asn::new(3)]);
    }

    #[test]
    fn test_get_mock() {
        let ap = AsPath::get_mock(None);
        assert_eq!(ap.len(), 3);
        assert_eq!(ap.get_asns(), &vec![Asn::new(3), Asn::new(2), Asn::new(1)]);
    }

    #[test]
    fn test_get_mock_custom_path() {
        let custom_path = vec![Asn::new(10), Asn::new(20), Asn::new(30)];
        let ap = AsPath::get_mock(Some(custom_path.clone()));
        assert_eq!(ap.get_asns(), &custom_path);
    }

    #[test]
    fn test_get_asns_empty() {
        let ap = AsPath::new(Vec::new());
        assert_eq!(ap.get_asns(), &Vec::new());
    }

    #[test]
    fn test_get_asns() {
        let as_path = vec![Asn::new(100), Asn::new(200), Asn::new(300)];
        let ap = AsPath::new(as_path.clone());
        assert_eq!(ap.get_asns(), &as_path);
    }

    #[test]
    fn test_is_empty_true() {
        let ap = AsPath::new(Vec::new());
        assert!(ap.is_empty());
    }

    #[test]
    fn test_is_empty_false() {
        let ap = AsPath::get_mock(None);
        assert!(!ap.is_empty());
    }

    #[test]
    fn test_len() {
        let ap = AsPath::new(vec![Asn::new(1), Asn::new(2), Asn::new(3)]);
        assert_eq!(ap.len(), 3);

        let ap_empty = AsPath::new(Vec::new());
        assert_eq!(ap_empty.len(), 0);
    }

    #[test]
    fn test_from_vec() {
        let bgpkit_asns = vec![
            BgpKitAsn::new_32bit(100),
            BgpKitAsn::new_32bit(200),
            BgpKitAsn::new_32bit(300),
        ];

        let ap = AsPath::from_vec(&bgpkit_asns);

        assert_eq!(ap.len(), 3);
        assert_eq!(
            ap.get_asns(),
            &vec![Asn::new(100), Asn::new(200), Asn::new(300)]
        );
    }

    #[test]
    fn test_from_vec_empty() {
        let bgpkit_asns: Vec<BgpKitAsn> = Vec::new();
        let ap = AsPath::from_vec(&bgpkit_asns);

        assert_eq!(ap.len(), 0);
        assert!(ap.is_empty());
    }

    #[test]
    fn test_serialize() {
        let ap = AsPath::new(vec![Asn::new(1), Asn::new(2), Asn::new(3)]);
        let json = serde_json::to_string(&ap).unwrap();
        assert!(json == "{\"as_path\":\"1,2,3\"}");
    }

    #[test]
    fn test_as_path_eq() {
        let asn_1 = Asn::new(1);

        // EQ with same default origin and default AS path
        let mut ap_1 = AsPath::get_mock(None);
        let mut ap_2 = AsPath::get_mock(None);
        assert_eq!(ap_1, ap_2);

        // EQ with same explicit origin and explicit AS path
        ap_1 = AsPath::get_mock(Some(vec![asn_1.clone()]));
        ap_2 = AsPath::get_mock(Some(vec![asn_1.clone()]));
        assert_eq!(ap_1, ap_2);
    }

    #[test]
    fn test_as_path_ne() {
        let asn_1 = Asn::new(1);
        let asn_2 = Asn::new(2);

        // NE with different origins
        let ap_1 = AsPath::get_mock(Some(vec![asn_1.clone()]));
        let ap_2 = AsPath::get_mock(Some(vec![asn_2.clone()]));
        assert_ne!(ap_1, ap_2);
    }
}
