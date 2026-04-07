pub mod path_filters {
    use crate::args::cli_args::CliArgs;
    use crate::clients::file::ensure_dir;
    use crate::clients::peeringdb::get_ixp_rs_asns;
    use crate::data::paths::Paths;
    use crate::types::asn::Asn;

    pub fn filter_results(paths: &mut Paths, args: &CliArgs) {
        ensure_dir(&args.results_dir);

        let filename = format!("{}/ixp_rs_asns.json", &args.results_dir);
        let _ixp_rs_asns = get_ixp_rs_asns(&args.peeringdb, &filename);

        let filename = format!("{}/divergent_paths.json", &args.results_dir);
        filter_paths(paths, &filename);

        let filename = format!("{}/has_unknown_community_asns.json", &args.results_dir);
        filter_with_unknown_community_asns(paths, &_ixp_rs_asns, &filename);

        let filename = format!("{}/only_unknown_community_asns.json", &args.results_dir);
        filter_only_unknown_community_asns(paths, &_ixp_rs_asns, &filename);
    }

    /// Remove all origins and paths which provide no indication of hidden ASNs in the path:
    /// * Single-hop AS paths
    /// * Paths that are not divergent
    /// * Origins with one or less AS paths
    fn filter_paths(paths: &mut Paths, filename: &String) {
        paths.print_summary();
        paths.remove_single_hop_as_paths();
        paths.print_summary();
        paths.remove_non_divergent_as_paths();
        paths.print_summary();
        paths.remove_origins_with_one_or_less_as_paths();
        paths.print_summary();
        paths.populate_diverging_asns();
        paths.to_file(filename);
    }

    /// Remove all origins and paths which provide no indication of hidden ASNs in the path based on community ASNs:
    /// * Paths with only known community ASNs
    ///
    /// This may result in some origins having one or no remaining AS paths.
    fn filter_with_unknown_community_asns(
        paths: &mut Paths,
        known_asns: &[Asn],
        filename: &String,
    ) {
        paths.remove_as_paths_with_only_known_community_asns(known_asns);
        paths.print_summary();
        paths.remove_origins_with_one_or_less_as_paths();
        paths.print_summary();
        paths.populate_diverging_asns();
        paths.to_file(filename);
    }

    /// Remove communities from the AS paths which have a known ASN
    fn filter_only_unknown_community_asns(
        paths: &mut Paths,
        known_asns: &[Asn],
        filename: &String,
    ) {
        paths.remove_communities_with_known_asns(known_asns);
        paths.print_summary();
        paths.to_file(filename);
    }
}
