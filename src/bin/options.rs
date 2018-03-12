use std::path::PathBuf;

#[derive(Debug, StructOpt)]
#[structopt(raw(setting = "::structopt::clap::AppSettings::ColoredHelp"))]
pub struct Options {
    /// The source file path.
    #[structopt(name = "FILE", parse(from_os_str))]
    pub path: PathBuf,

    /// Turns off message output.
    #[structopt(short = "q", long = "quiet")]
    pub quiet: bool,

    /// Increases the verbosity. Default verbosity is errors only.
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    pub verbose: usize,
}

impl Options {
    /// Sets up logging as specified by the `-q` and `-v` flags.
    pub fn start_logger(&self) {
        if !self.quiet {
            let r = ::stderrlog::new().verbosity(1 + self.verbose).init();
            if let Err(err) = r {
                eprintln!("Warning: logging couldn't start: {}", err);
            }
        }
    }
}
