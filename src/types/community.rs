use crate::types::asn::Asn;
use bgpkit_parser::models::Community;
use serde::ser::SerializeTuple;
use serde::{Serialize, Serializer};
use std::hash::Hash;

/// Standard community which can be serialised to JSON.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StandardCommunity {
    asn: Asn,
    value: u16,
}

impl Serialize for StandardCommunity {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut tup = serializer.serialize_tuple(2)?;
        tup.serialize_element(&self.get_asn().clone().to_u32())?;
        tup.serialize_element(&self.get_value())?;
        tup.end()
    }
}

impl Hash for StandardCommunity {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.asn.clone().to_u32().hash(state);
        self.value.hash(state);
    }
}

impl StandardCommunity {
    pub fn new(asn: Asn, value: u16) -> Self {
        Self { asn, value }
    }

    pub fn from_community(community: &Community) -> Self {
        if let Community::Custom(asn, value) = community {
            StandardCommunity::new(Asn::new(asn.to_u32()), *value)
        } else if let Community::NoAdvertise = community {
            StandardCommunity::new(Asn::new(65535), 65281)
        } else if let Community::NoExport = community {
            StandardCommunity::new(Asn::new(65535), 65282)
        } else if let Community::NoExportSubConfed = community {
            StandardCommunity::new(Asn::new(65535), 65283)
        } else {
            panic!(
                "Couldn't unpack Community into StandardCommunity: {}",
                community
            );
        }
    }

    pub fn get_mock(community: Option<(Asn, u16)>) -> StandardCommunity {
        let parts = community.unwrap_or((Asn::get_mock(None), 23456));
        StandardCommunity::new(parts.0, parts.1)
    }

    pub fn get_asn(&self) -> &Asn {
        &self.asn
    }

    fn get_value(&self) -> &u16 {
        &self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_new() {
        let test_cases = vec![
            (Asn::new(0), 0),
            (Asn::new(1), 1),
            (Asn::new(65535), 65535),
            (Asn::new(4294967295), 12345),
        ];

        for (asn, value) in test_cases {
            let community = StandardCommunity::new(asn.clone(), value);
            assert_eq!(community.get_asn(), &asn);
            assert_eq!(community.get_value(), &value);
        }
    }

    #[test]
    fn test_from_community_custom() {
        let bgpkit_asn = bgpkit_parser::models::Asn::new_32bit(64512);
        let community = Community::Custom(bgpkit_asn, 100);

        let std_community = StandardCommunity::from_community(&community);

        assert_eq!(std_community.get_asn().clone().to_u32(), 64512);
        assert_eq!(std_community.get_value(), &100);
    }

    #[test]
    fn test_from_community_no_advertise() {
        let community = Community::NoAdvertise;
        let std_community = StandardCommunity::from_community(&community);

        assert_eq!(std_community.get_asn().clone().to_u32(), 65535);
        assert_eq!(std_community.get_value(), &65281);
    }

    #[test]
    fn test_from_community_no_export() {
        let community = Community::NoExport;
        let std_community = StandardCommunity::from_community(&community);

        assert_eq!(std_community.get_asn().clone().to_u32(), 65535);
        assert_eq!(std_community.get_value(), &65282);
    }

    #[test]
    fn test_from_community_no_export_sub_confed() {
        let community = Community::NoExportSubConfed;
        let std_community = StandardCommunity::from_community(&community);

        assert_eq!(std_community.get_asn().clone().to_u32(), 65535);
        assert_eq!(std_community.get_value(), &65283);
    }

    #[test]
    fn test_get_mock_with_none() {
        let community = StandardCommunity::get_mock(None);
        assert_eq!(
            community.get_asn().clone().to_u32(),
            Asn::get_mock(None).to_u32()
        );
        assert_eq!(community.get_value(), &23456);
    }

    #[test]
    fn test_get_mock_with_some() {
        let asn = Asn::new(12345);
        let value = 54321;
        let community = StandardCommunity::get_mock(Some((asn.clone(), value)));
        assert_eq!(community.get_asn(), &asn);
        assert_eq!(community.get_value(), &value);
    }

    #[test]
    fn test_get_asn() {
        let asn = Asn::new(64512);
        let community = StandardCommunity::new(asn.clone(), 100);
        assert_eq!(community.get_asn(), &asn);
    }

    #[test]
    fn test_get_value() {
        let community = StandardCommunity::new(Asn::new(64512), 100);
        assert_eq!(community.get_value(), &100);
    }

    #[test]
    fn test_serialize() {
        let test_cases = vec![
            (0, 0, "[0,0]"),
            (1, 1, "[1,1]"),
            (65535, 65535, "[65535,65535]"),
            (4294967295, 12345, "[4294967295,12345]"),
        ];

        for (asn, value, expected) in test_cases {
            let community = StandardCommunity::new(Asn::new(asn), value);
            let json = serde_json::to_string(&community).unwrap();
            assert_eq!(json, expected);
        }
    }

    #[test]
    fn test_serialize_in_vec() {
        let communities = vec![
            StandardCommunity::new(Asn::new(100), 200),
            StandardCommunity::new(Asn::new(300), 400),
        ];

        let json = serde_json::to_string(&communities).unwrap();
        assert_eq!(json, "[[100,200],[300,400]]");
    }

    #[test]
    fn test_clone() {
        let community1 = StandardCommunity::new(Asn::new(64512), 100);
        let community2 = community1.clone();
        assert_eq!(community1, community2);
    }

    #[test]
    fn test_partial_eq() {
        let community1 = StandardCommunity::new(Asn::new(64512), 100);
        let community2 = StandardCommunity::new(Asn::new(64512), 100);
        let community3 = StandardCommunity::new(Asn::new(64512), 200);
        let community4 = StandardCommunity::new(Asn::new(65000), 100);
        assert_eq!(community1, community2);
        assert_ne!(community1, community3);
        assert_ne!(community1, community4);
    }

    #[test]
    fn test_hash() {
        let community1 = StandardCommunity::new(Asn::new(64512), 100);
        let community2 = StandardCommunity::new(Asn::new(64512), 100);
        let community3 = StandardCommunity::new(Asn::new(64512), 200);

        let mut set = HashSet::new();
        set.insert(community1.clone());

        assert!(set.contains(&community2));
        assert!(!set.contains(&community3));
    }

    #[test]
    fn test_debug() {
        let community = StandardCommunity::new(Asn::new(64512), 100);
        let debug_str = format!("{:?}", community);
        assert!(debug_str.contains("StandardCommunity"));
        assert!(debug_str.contains("asn"));
        assert!(debug_str.contains("value"));
        assert!(debug_str == "StandardCommunity { asn: Asn(64512), value: 100 }");
    }
}
