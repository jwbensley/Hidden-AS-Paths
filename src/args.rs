pub mod cli_args {
    use clap::{Args, Parser};

    #[derive(Debug, Args, Clone)]
    pub struct DownloadArgs {
        /// Download RIBs to this directory
        #[arg(short = 'p', long, default_value_t = String::from("./mrts"))]
        pub ribs_path: String,

        /// Download RIBs for yyyy-mm-dd
        #[arg(short = 'y', long, default_value_t = String::from("2025-09-22"))]
        pub ribs_ymd: String,
    }

    #[derive(Debug, Args)]
    #[group(required = true, multiple = false)]
    pub struct RibsSource {
        #[command(flatten)]
        pub download_args: DownloadArgs,

        /// Glob of MRT files to parse (if not specified, all MRT files for specified date will be scanned)
        #[arg(short = 'f', long, value_delimiter = ' ', num_args = 1..)]
        pub rib_files: Vec<String>,
    }

    /// Scan MRT RIB dumps, looking for potential instances of ASN hidding
    #[derive(Parser, Debug)]
    #[command(version, about, long_about = None)]
    pub struct CliArgs {
        /// Run with debug level logging
        #[arg(short, long)]
        pub debug: bool,

        #[command(flatten)]
        pub ribs_source: RibsSource,

        /// Number of threads to use for parsing MRT files
        #[arg(short, long, default_value_t = 1)]
        pub threads: usize,
    }

    impl CliArgs {
        pub fn get_ribs_path(&self) -> &str {
            self.ribs_source.download_args.ribs_path.as_str()
        }

        pub fn get_ribs_ymd(&self) -> &str {
            self.ribs_source.download_args.ribs_ymd.as_str()
        }
    }

    pub fn parse_cli_arg() -> CliArgs {
        CliArgs::parse()
    }
}
