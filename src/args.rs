pub mod cli_args {
    use clap::{Args, Parser, Subcommand};

    /// Download RIB files by specifying an output folder and a date.
    /// The downloaded files will then be parsed (existing files are not re-downloaded).
    #[derive(Debug, Args)]
    pub struct DownloadArgs {
        /// Download RIBs to this directory
        #[arg(short = 'p', long, default_value_t = String::from("/opt/mrts"))]
        pub ribs_path: String,

        /// Download RIBs at 00.00 for given yyyy-mm-dd
        #[arg(short = 'y', long, default_value_t = String::from("2026-01-14"))]
        pub ribs_ymd: String,
    }

    /// Parse a RIB file which already exists locally - split across multiple thread.
    #[derive(Debug, Args)]
    pub struct FileArgs {
        /// Path to an existing MRT file to parse
        #[arg(short = 'f', long)]
        pub rib_file: String,
    }

    /// Parse RIB files which already exist locally - each file by a different thread.
    #[derive(Debug, Args)]
    pub struct FilesArgs {
        /// Space separated list of existing MRT files to parse, a BASH glob will be expanded into a space separated list by the shell
        #[arg(short = 'f', long, value_delimiter = ' ', num_args = 1..)]
        pub rib_files: Vec<String>,
    }

    #[derive(Subcommand, Debug)]
    pub enum RibsSource {
        Download(DownloadArgs),
        File(FileArgs),
        Files(FilesArgs),
    }

    /// Scan MRT RIB dumps, looking for AS paths which suggest an ASN has been stripped from the path.
    #[derive(Parser, Debug)]
    #[command(version, about, long_about = None)]
    pub struct CliArgs {
        /// Run with debug level logging
        #[arg(short, long)]
        pub debug: bool,

        #[command(subcommand)]
        pub ribs_source: RibsSource,

        /// Serialise found AS paths to a JSON file
        #[arg(long, default_value_t = String::from("./paths.json"))]
        pub paths: String,

        /// Number of threads to use for parsing files
        #[arg(short, long, default_value_t = 1)]
        pub threads: u32,
    }

    impl CliArgs {
        pub fn get_ribs_path(&self) -> &str {
            if let RibsSource::Download(args) = &self.ribs_source {
                args.ribs_path.as_str()
            } else {
                panic!("No RIB path option to unpack");
            }
        }

        pub fn get_ribs_ymd(&self) -> &str {
            if let RibsSource::Download(args) = &self.ribs_source {
                args.ribs_ymd.as_str()
            } else {
                panic!("No RIB ymd option to unpack");
            }
        }

        pub fn get_rib_file(&self) -> &String {
            if let RibsSource::File(args) = &self.ribs_source {
                &args.rib_file
            } else {
                panic!("No RIB file option to unpack");
            }
        }

        pub fn get_rib_files(&self) -> &Vec<String> {
            if let RibsSource::Files(args) = &self.ribs_source {
                &args.rib_files
            } else {
                panic!("No RIB file list option to unpack");
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
