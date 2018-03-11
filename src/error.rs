use std::io::Error as IoError;

/// An error from `oftb`.
#[derive(Debug, Fail)]
pub enum Error {
    /// An error opening a source file.
    #[fail(display = "Couldn't open `{:?}': {}", _0, _1)]
    CouldntOpenSource(String, IoError),

    /// A parse error.
    #[fail(display = "Syntax error in {}\n{}", _0, _1)]
    Parse(String, String),
}
