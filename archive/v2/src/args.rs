pub mod cli_args {
    use clap::{Args, Parser, Subcommand};

    /// Download RIB files by specifying an output folder and a date.
    /// The downloaded files will then be parsed (existing files are not re-downloaded).
    #[derive(Debug, Args)]
    pub struct DownloadArgs {
        /// Download RIBs to this directory
        #[arg(short = 'p', long, default_value_t = String::from("./mrts/"))]
        pub ribs_path: String,

        /// Download RIBs for yyyy-mm-dd
        #[arg(short = 'y', long, default_value_t = String::from("2025-09-22"))]
        pub ribs_ymd: String,
    }

    /// Parse RIB files which a;ready exist locally.
    #[derive(Debug, Args)]
    pub struct FileArgs {
        /// Space seperated list of existing MRT files to parse
        #[arg(short = 'f', long, value_delimiter = ' ', num_args = 1..)]
        pub rib_files: Vec<String>,
    }

    /// Print the specific record from an MRT file (by index number)
    #[derive(Debug, Args)]
    pub struct PrintArgs {
        /// Record/entry index in MRT file to print
        #[arg(short = 'i', long)]
        pub mrt_index: u32,

        /// Existing MRT files to parse
        #[arg(short = 'f', long)]
        pub rib_file: String,
    }

    #[derive(Subcommand, Debug)]
    pub enum RibsSource {
        Download(DownloadArgs),
        File(FileArgs),
        Print(PrintArgs),
    }

    /// Scan MRT RIB dumps, looking for potential instances of ASN hiding
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
        pub threads: u32,
    }

    impl CliArgs {
        pub fn get_mrt_index(&self) -> &u32 {
            if let RibsSource::Print(args) = &self.ribs_source {
                &args.mrt_index
            } else {
                panic!("No CLI option to unpack");
            }
        }

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

        pub fn get_rib_file(&self) -> &String {
            if let RibsSource::Print(args) = &self.ribs_source {
                &args.rib_file
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

        pub fn print(&self) -> bool {
            matches!(self.ribs_source, RibsSource::Print(_))
        }
    }

    pub fn parse_cli_arg() -> CliArgs {
        CliArgs::parse()
    }
}
