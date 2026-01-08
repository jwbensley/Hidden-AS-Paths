pub mod cli_args {
    use clap::{Args, Parser, Subcommand};

    /// Download RIB files by specifying an output folder and a date.
    /// The downloaded files will then be parsed (existing files are not re-downloaded).
    #[derive(Debug, Args)]
    pub struct DownloadArgs {
        /// Download RIBs to this directory
        #[arg(short = 'p', long, default_value_t = String::from("./mrts"))]
        pub ribs_path: String,

        /// Download RIBs for yyyy-mm-dd
        #[arg(short = 'y', long, default_value_t = String::from("2025-09-22"))]
        pub ribs_ymd: String,
    }

    /// Parse RIB files which aready exist locally.
    #[derive(Debug, Args)]
    pub struct FileArgs {
        /// Space seperated list of existing MRT files to parse
        #[arg(short = 'f', long, value_delimiter = ' ', num_args = 1..)]
        pub rib_files: Vec<String>,
    }

    #[derive(Subcommand, Debug)]
    pub enum RibsSource {
        Download(DownloadArgs),
        File(FileArgs),
    }

    /// Scan MRT RIB dumps, looking for potential instances of ASN hidding
    #[derive(Parser, Debug)]
    #[command(version, about, long_about = None)]
    pub struct CliArgs {
        /// Run with debug level logging
        #[arg(short, long)]
        pub debug: bool,

        #[command(subcommand)]
        pub ribs_source: RibsSource,

        /// Number of threads to use for parsing MRT files
        #[arg(short, long, default_value_t = 1)]
        pub threads: usize,
    }

    impl CliArgs {
        pub fn get_ribs_path(&self) -> &str {
            if let RibsSource::Download(args) = &self.ribs_source {
                args.ribs_path.as_str()
            } else {
                panic!("No CLI option to unpack");
            }
        }

        pub fn get_ribs_ymd(&self) -> &str {
            if let RibsSource::Download(args) = &self.ribs_source {
                args.ribs_ymd.as_str()
            } else {
                panic!("No CLI option to unpack");
            }
        }

        pub fn get_rib_files(&self) -> &Vec<String> {
            if let RibsSource::File(args) = &self.ribs_source {
                &args.rib_files
            } else {
                panic!("No CLI option to unpack");
            }
        }

        pub fn download(&self) -> bool {
            matches!(self.ribs_source, RibsSource::Download(_))
        }
    }

    pub fn parse_cli_arg() -> CliArgs {
        CliArgs::parse()
    }
}
