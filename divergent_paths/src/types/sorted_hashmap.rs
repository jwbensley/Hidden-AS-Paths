use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};
use std::collections::HashMap;

/// Stolen from https://www.codestudy.net/blog/how-to-sort-hashmap-keys-when-serializing-with-serde/

#[derive(Debug, Clone)]
pub struct SortedHashMap<'a, K, V>(pub &'a HashMap<K, V>);

impl<'a, K, V> Serialize for SortedHashMap<'a, K, V>
where
    K: Serialize + Ord, // Keys must be sortable (Ord) and serializable
    V: Serialize,       // Values must be serializable
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Collect HashMap entries into a Vec for sorting
        let mut entries: Vec<_> = self.0.iter().collect();

        // Sort entries by key (ascending order by default)
        entries.sort_by_key(|&(key, _)| key);

        // Serialize as a map: start the map, write sorted entries, end the map
        let mut map = serializer.serialize_map(Some(entries.len()))?;
        for (key, value) in entries {
            map.serialize_entry(key, value)?; // Serialize key-value pair
        }
        map.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::to_string;

    #[test]
    fn test_sorted_serialization() {
        let mut map = HashMap::new();
        map.insert(3, "three");
        map.insert(1, "one");
        map.insert(2, "two");

        let sorted_map = SortedHashMap(&map);
        let json = to_string(&sorted_map).unwrap();

        // Keys should be sorted numerically: 1, 2, 3
        assert!(json.contains(r#"{"1":"one","2":"two","3":"three"}"#));
    }

    #[test]
    fn test_empty_map() {
        let map: HashMap<&str, i32> = HashMap::new();
        let sorted_map = SortedHashMap(&map);
        let json = to_string(&sorted_map).unwrap();
        assert_eq!(json, "{}");
    }
}
