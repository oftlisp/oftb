use std::env::var_os;
use std::path::PathBuf;
use std::process::exit;

#[derive(Debug, StructOpt)]
#[structopt(raw(setting = "::structopt::clap::AppSettings::ColoredHelp"))]
pub struct Options {
    /// The path to the main package.
    #[structopt(name = "PACKAGE-PATH", parse(from_os_str))]
    pub package_path: PathBuf,

    /// The binary to compile.
    #[structopt(name = "BINARY-NAME")]
    pub binary_name: String,

    /// The path to write the output file to.
    #[structopt(short = "o", long = "output", name = "OUTPUT-PATH",
                parse(from_os_str))]
    pub output_path: Option<PathBuf>,

    /// The path to the `std` package. If not present, defaults to
    /// `$OFTLISP_ROOT/std`.
    #[structopt(long = "std", name = "PATH", parse(from_os_str))]
    pub std_path: Option<PathBuf>,

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
                error!("Warning: logging couldn't start: {}", err);
            }
        }
    }

    /// Returns the path to write the output file to.
    pub fn output_path(&self) -> PathBuf {
        match self.output_path {
            Some(ref path) => path.clone(),
            None => PathBuf::from(format!("{}.ofta", self.binary_name)),
        }
    }

    /// Gets the path of the `std` package.
    pub fn std_path(&self) -> PathBuf {
        match self.std_path.as_ref() {
            Some(path) => path.as_path().into(),
            None => match var_os("OFTLISP_ROOT") {
                Some(path) => {
                    let mut path = PathBuf::from(path);
                    path.push("std");
                    path.into()
                }
                None => {
                    error!("Can't find the standard library; either pass --std or define the OFTLISP_ROOT environment variable");
                    exit(1);
                }
            },
        }
    }
}
