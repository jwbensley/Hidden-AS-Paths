pub mod searches {

    use crate::asp_tree::asp_trees::{AsSequences, Route};
    use bgpkit_parser::models::{Asn, Community};
    use log::{debug, info};

    // Search for communities which below to ASNs not in the AS path
    pub fn communities_without_asn(as_sequences: &AsSequences) {
        let mut found_routes = Vec::<Route>::new();

        for sequence in as_sequences.get_sequences() {
            let as_paths = as_sequences.get_as_paths_at_sequence(sequence);
            for as_path in as_paths.get_paths() {
                let routes = as_paths.get_routes_at_path(as_path);
                for route in routes.get_routes() {
                    let mut found = false;
                    for community in route.get_communities() {
                        if let Community::Custom(asn, _) = *community {
                            if asn.is_reserved() {
                                continue;
                            }

                            if !as_path.contains(&asn) {
                                info!("{:?}", sequence);
                                info!("    {:?}", as_path);
                                info!("Missing: {}", asn);
                                info!("        {:#?}", route);
                                info!("");
                                found = true;
                                found_routes.push(route.clone());
                                break;
                            }
                        }
                    }
                    if found {
                        continue;
                    }
                    for large_community in route.get_large_communities() {
                        let asn = Asn::new_32bit(large_community.global_admin);
                        if asn.is_reserved() {
                            continue;
                        }

                        if !as_path.contains(&asn) {
                            info!("{:?}", sequence);
                            info!("    {:?}", as_path);
                            info!("Missing: {}", asn);
                            info!("        {:#?}", route);
                            info!("");
                            found_routes.push(route.clone());
                            break;
                        }
                    }
                }
            }
        }
    }
}
