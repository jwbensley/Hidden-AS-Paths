pub mod route {
    use crate::mrt_asn::asn::Testing as AsnTesting;
    //////// use crate::mrt_community::communities::get_mock as get_mock_communities;
    use crate::mrt_ip_addr::ip_addr::Testing as IpAddrTesting;
    use crate::mrt_ip_net::ip_net::Testing as IpNetTesting;
    //////// use crate::mrt_large_community::large_communities::get_mock as get_mock_large_communities;
    use crate::mrt_peer::peer::Testing as PeerTesting;
    use bgpkit_parser::models::{Asn, Community, LargeCommunity, Peer};
    use ipnet::IpNet;
    use std::hash::Hash;
    use std::net::IpAddr;

    /// Store a route pulled from an MRT file (one route object per prefix)
    #[derive(Clone, Debug, Eq)]
    pub struct Route {
        as_path: Vec<Asn>,
        filename: String,
        next_hop: IpAddr,
        peer: Peer,
        prefix: IpNet,
        // communities: Vec<Community>,
        // large_communities: Vec<LargeCommunity>,
    }

    impl PartialEq for Route {
        fn eq(&self, other: &Self) -> bool {
            (self.as_path == other.as_path)
                && (self.filename == other.filename)
                && (self.next_hop == other.next_hop)
                && (self.peer == other.peer)
                && (self.prefix == other.prefix)
            // && (self.communities == other.communities)
            // && (self.large_communities == other.large_communities)
        }
    }

    impl Hash for Route {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            self.as_path.hash(state);
            self.filename.hash(state);
            self.next_hop.hash(state);
            self.peer.hash(state);
            self.prefix.hash(state);
        }
    }

    impl Route {
        pub fn new(
            as_path: Vec<Asn>,
            filename: String,
            next_hop: IpAddr,
            peer: Peer,
            prefix: IpNet,
            // communities: Vec<Community>,
            // large_communities: Vec<LargeCommunity>,
        ) -> Self {
            Self {
                as_path,
                filename,
                next_hop,
                peer,
                prefix,
                // communities,
                // large_communities,
            }
        }

        pub fn get_mock(origin: Option<Asn>) -> Route {
            let as_path = Vec::from([
                Asn::get_mock(Some(1)),
                Asn::get_mock(Some(2)),
                origin.unwrap_or(Asn::get_mock(None)),
            ]);

            Route {
                as_path,
                filename: String::from("unit test"),
                next_hop: IpAddr::get_mock(),
                peer: Peer::get_mock(),
                prefix: IpNet::get_mock(),
                // communities: get_mock_communities(None),
                // large_communities: get_mock_large_communities(None),
            }
        }

        pub fn get_as_path(&self) -> &Vec<Asn> {
            &self.as_path
        }

        // pub fn get_communities(&self) -> &Vec<Community> {
        //     &self.communities
        // }

        // pub fn get_large_communities(&self) -> &Vec<LargeCommunity> {
        //     &self.large_communities
        // }

        pub fn get_origin(&self) -> &Asn {
            self.as_path.last().unwrap()
        }
    }
}
