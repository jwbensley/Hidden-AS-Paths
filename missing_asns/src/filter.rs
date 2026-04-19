pub mod path_filters {
    use crate::args::cli_args::CliArgs;
    use crate::clients::file::ensure_dir;
    use crate::data::paths::Paths;

    pub fn filter_results(paths: &mut Paths, args: &CliArgs) {
        ensure_dir(&args.results_dir);

        let filename = format!("{}/as_paths.json", &args.results_dir);
        paths.to_file(&filename);
    }
}
