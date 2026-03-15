/// This module contains the StandardCommunity and StandardCommunities structs so that they can be serialised to JSON.
pub mod standard_communities {
    use crate::mrt_asn::asn::MrtAsn;
    use bgpkit_parser::models::Community;
    use serde::ser::{SerializeSeq, SerializeTuple};
    use serde::{Serialize, Serializer};
    use std::hash::Hash;
    use std::vec::Vec;

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct StandardCommunity {
        asn: MrtAsn,
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
        pub fn new(asn: MrtAsn, value: u16) -> Self {
            Self { asn, value }
        }

        fn get_mock(community: Option<(MrtAsn, u16)>) -> StandardCommunity {
            let parts = community.unwrap_or((MrtAsn::get_mock(None), 23456));
            StandardCommunity::new(parts.0, parts.1)
        }

        pub fn get_asn(&self) -> &MrtAsn {
            &self.asn
        }

        fn get_value(&self) -> &u16 {
            &self.value
        }
    }

    #[derive(Clone, Debug, Eq, PartialEq, Hash)]
    pub struct StandardCommunities {
        standard_communities: Vec<StandardCommunity>,
    }

    impl Serialize for StandardCommunities {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let mut seq = serializer.serialize_seq(Some(self.standard_communities.len()))?;
            for e in &self.standard_communities {
                seq.serialize_element(e)?;
            }
            seq.end()
        }
    }

    impl<'a> StandardCommunities {
        pub fn new(standard_communities: Vec<StandardCommunity>) -> Self {
            Self {
                standard_communities,
            }
        }

        pub fn default() -> Self {
            Self::new(Vec::<StandardCommunity>::new())
        }

        pub fn add(&mut self, c: StandardCommunity) {
            self.standard_communities.push(c);
        }

        pub fn from_vec(communities: Vec<Community>) -> Self {
            let mut standard_communities = Self::default();
            for community in communities {
                if let Community::Custom(asn, value) = community {
                    standard_communities
                        .add(StandardCommunity::new(MrtAsn::new(asn.to_u32()), value));
                } else if let Community::NoAdvertise = community {
                    standard_communities.add(StandardCommunity::new(MrtAsn::new(65535), 65281));
                } else if let Community::NoExport = community {
                    standard_communities.add(StandardCommunity::new(MrtAsn::new(65535), 65282));
                } else if let Community::NoExportSubConfed = community {
                    standard_communities.add(StandardCommunity::new(MrtAsn::new(65535), 65283));
                    // NO-OP - we're not interested in these communities
                } else {
                    panic!(
                        "Couldn't unpack Community into StandardCommunity: {}",
                        community
                    );
                }
            }
            standard_communities
        }
    }
}
