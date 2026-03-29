use bgpkit_parser::models::Asn as BgpkitAsn;
use serde::{Serialize, Serializer};
use std::fmt;

/// A wrapper around the `Asn` type from `bgpkit_parser` to allow for serialisation to JSON.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Asn(BgpkitAsn);

impl fmt::Display for Asn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for Asn {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u32(self.0.to_u32())
    }
}

impl Asn {
    pub fn new(asn: u32) -> Self {
        Self(BgpkitAsn::new_32bit(asn))
    }

    pub fn get_mock(asn: Option<u32>) -> Asn {
        Asn::new(asn.unwrap_or(1))
    }

    pub fn is_private(&self) -> bool {
        if self.0 == 0 {
            // RFC 7607
            true
        } else if self.0 == 23456 {
            // RFC 4893
            true
        } else if (64496..=64511).contains(&self.0.to_u32()) {
            // RFC 5398
            true
        } else if (64512..=65535).contains(&self.0.to_u32()) {
            // RFC 6996
            true
        } else if (65536..=65551).contains(&self.0.to_u32()) {
            // RFC 5398
            true
        } else if (65552..131071).contains(&self.0.to_u32()) {
            // IANA reserved
            true
        } else if (4200000000..=4294967295).contains(&self.0.to_u32()) {
            // RFC 6996
            true
        } else {
            false
        }
    }

    pub fn to_u32(self) -> u32 {
        self.0.to_u32()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let as_numbers = vec![0, 1, 65535, 4294967295];
        for as_number in as_numbers {
            let asn = Asn::new(as_number);
            assert_eq!(asn.to_u32(), as_number);
        }
    }

    #[test]
    fn test_get_mock() {
        let asn = Asn::get_mock(None);
        assert_eq!(asn.to_u32(), 1);

        let asn = Asn::get_mock(Some(12345));
        assert_eq!(asn.to_u32(), 12345);
    }

    #[test]
    fn test_to_u32() {
        let asn = Asn::new(54321);
        let asn_u32: u32 = asn.to_u32();
        assert_eq!(asn_u32, 54321);
    }

    #[test]
    fn test_is_private() {
        let private_asns = vec![
            0, 23456, 64496, 64511, 64512, 65535, 65536, 65551, 65552, 131070, 4200000000,
            4294967295,
        ];
        for as_number in private_asns {
            assert!(
                Asn::new(as_number).is_private(),
                "ASN {} should be private",
                as_number
            );
        }

        let public_asns = vec![1, 64495, 131072, 4199999999];
        for as_number in public_asns {
            assert!(
                !Asn::new(as_number).is_private(),
                "ASN {} should be public",
                as_number
            );
        }
    }

    #[test]
    fn test_display() {
        let asn = Asn::new(65000);
        assert_eq!(format!("{}", asn), "65000");
    }

    #[test]
    fn test_serialize() {
        let asn = Asn::new(12345);
        let json = serde_json::to_string(&asn).unwrap();
        assert_eq!(json, "12345");
    }

    #[test]
    fn test_serialize_in_struct() {
        #[derive(Serialize)]
        struct TestStruct {
            asn: Asn,
        }

        let test = TestStruct {
            asn: Asn::new(64512),
        };
        let json = serde_json::to_string(&test).unwrap();
        assert_eq!(json, r#"{"asn":64512}"#);
    }
}
