use std::fs::File;
use std::io::Read;
use std::path::Path;

use failure::{err_msg, ResultExt};
use pest::Parser;

use error::{Error, ErrorKind};
use literal::Literal;

mod convert;
mod symbolish;

#[cfg(test)]
mod tests;

#[cfg(debug_assertions)]
const _GRAMMAR: &'static str = include_str!("../oftlisp.pest");

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
        .with_context(|_| ErrorKind::CouldntOpenSource(path.display().to_string()))?;
    debug!("Parsing `{}'...", path.display());
    parse_program(&src)
        .map_err(|e| err_msg(e.to_string()))
        .with_context(|_| ErrorKind::Parse(path.display().to_string()))
        .map_err(|e| e.into())
}

/// Parses OftLisp source code.
pub fn parse_program<'src>(src: &'src str) -> Result<Vec<Literal>, ::pest::Error<'src, Rule>> {
    let pairs = OftLispParser::parse(Rule::program, src)?;
    debug!("Finished parsing, converting to Literals...");
    convert::convert_program(pairs)
}
