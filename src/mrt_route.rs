pub mod route {
    use bgpkit_parser::models::{Asn, Community, LargeCommunity, Peer};
    use ipnet::IpNet;
    use std::net::IpAddr;

    /// Store a route pulled from an MRT file (one route object per prefix)
    #[derive(Clone, Debug)]
    pub struct Route {
        as_path: Vec<Asn>,
        filename: String,
        next_hop: IpAddr,
        peer: Peer,
        prefix: IpNet,
        communities: Vec<Community>,
        large_communities: Vec<LargeCommunity>,
    }

    impl PartialEq for Route {
        fn eq(&self, other: &Self) -> bool {
            (self.as_path == other.as_path)
                && (self.filename == other.filename)
                && (self.next_hop == other.next_hop)
                && (self.peer == other.peer)
                && (self.prefix == other.prefix)
        }
    }

    impl Route {
        pub fn new(
            as_path: Vec<Asn>,
            filename: String,
            next_hop: IpAddr,
            peer: Peer,
            prefix: IpNet,
            communities: Vec<Community>,
            large_communities: Vec<LargeCommunity>,
        ) -> Self {
            Self {
                as_path,
                filename,
                next_hop,
                peer,
                prefix,
                communities,
                large_communities,
            }
        }

        pub fn get_as_path(&self) -> &Vec<Asn> {
            &self.as_path
        }

        pub fn get_communities(&self) -> &Vec<Community> {
            &self.communities
        }

        pub fn get_large_communities(&self) -> &Vec<LargeCommunity> {
            &self.large_communities
        }

        pub fn get_origin(&self) -> &Asn {
            self.as_path.last().unwrap()
        }
    }
}
