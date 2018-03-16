use std::fs::File;
use std::io::Read;
use std::path::Path;

use pest::Parser;

use error::Error;
use literal::Literal;

mod convert;
#[cfg(test)]
mod tests;

#[derive(Parser)]
#[grammar = "oftlisp.pest"]
pub struct OftLispParser;

/// Parses an OftLisp program from a file. If successful, returns the values as
/// Literals.
pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<Vec<Literal>, Error> {
    let path = path.as_ref();
    debug!("Loading `{}'...", path.display());
    let src = File::open(path)
        .and_then(|mut file| {
            let mut src = String::new();
            file.read_to_string(&mut src).map(|_| src)
        })
        .map_err(|err| {
            Error::CouldntOpenSource(path.display().to_string(), err)
        })?;
    parse_program(&src).map_err(|err| {
        Error::Parse(path.display().to_string(), err.to_string())
    })
}

/// Parses OftLisp source code.
pub fn parse_program<'src>(
    src: &'src str,
) -> Result<Vec<Literal>, ::pest::Error<'src, Rule>> {
    OftLispParser::parse(Rule::program, src).and_then(convert::convert_program)
}
