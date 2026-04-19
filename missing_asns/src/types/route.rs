use crate::types::community::StandardCommunity;
use ipnet::IpNet;
use serde::ser::SerializeStruct as _;
use serde::{Serialize, Serializer};
use std::hash::Hash;

/// Store a route pulled from an MRT file and provide serialisation to JSON.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Route {
    filename: String,
    prefix: IpNet,
    communities: Vec<StandardCommunity>,
}

impl Serialize for Route {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Route", 3)?;
        state.serialize_field("filename", &self.filename)?;
        state.serialize_field("prefix", &self.prefix.to_string())?;
        state.serialize_field("communities", &self.communities)?;
        state.end()
    }
}

impl Route {
    pub fn new(filename: String, prefix: IpNet, communities: Vec<StandardCommunity>) -> Self {
        Self {
            filename,
            prefix,
            communities,
        }
    }

    pub fn get_mock() -> Self {
        Self {
            filename: String::from("mock_filename"),
            prefix: "127.0.0.0/8".parse().unwrap(),
            communities: Vec::from([StandardCommunity::get_mock(None)]),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::types::asn::Asn;

    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_new() {
        let filename = String::from("test_file.mrt");
        let prefix: IpNet = "10.0.0.0/8".parse().unwrap();
        let communities = vec![
            StandardCommunity::new(Asn::new(12345), 100),
            StandardCommunity::new(Asn::new(54321), 200),
        ];
        let route = Route::new(filename.clone(), prefix, communities);
        assert_eq!(route.filename, filename);
        assert_eq!(route.prefix, prefix);
        assert_eq!(
            route.communities,
            vec![
                StandardCommunity::new(Asn::new(12345), 100),
                StandardCommunity::new(Asn::new(54321), 200),
            ]
        );
    }

    #[test]
    fn test_get_mock() {
        let route = Route::get_mock();
        assert_eq!(route.filename, "mock_filename");
        assert_eq!(route.prefix, "127.0.0.0/8".parse::<IpNet>().unwrap());
    }

    #[test]
    fn test_serialize() {
        let route = Route::new(
            String::from("test.mrt"),
            "10.0.0.0/8".parse().unwrap(),
            vec![StandardCommunity::new(Asn::new(12345), 100)],
        );
        let json = serde_json::to_string(&route).unwrap();
        let expected_json = "{\"filename\":\"test.mrt\",\"prefix\":\"10.0.0.0/8\",\"communities\":[{\"asn\":12345,\"value\":100}]}";
        assert!(json == expected_json);
    }

    #[test]
    fn test_clone() {
        let route1 = Route::get_mock();
        let route2 = route1.clone();
        assert_eq!(route1, route2);
    }

    #[test]
    fn test_partial_eq() {
        let route1 = Route::get_mock();
        let route2 = Route::get_mock();

        assert_eq!(route1, route2);

        let route3 = Route::new(
            String::from("different.mrt"),
            "192.0.2.2".parse().unwrap(),
            vec![],
        );

        assert_ne!(route1, route3);
    }

    #[test]
    fn test_hash() {
        let route1 = Route::get_mock();
        let route2 = Route::get_mock();

        let mut set = HashSet::new();
        set.insert(route1.clone());

        assert!(set.contains(&route2));

        let route3 = Route::new(
            String::from("different.mrt"),
            "192.0.2.2".parse().unwrap(),
            vec![],
        );

        assert!(!set.contains(&route3));
    }
}
