use std::fs::File;
use std::io::Read;
use std::path::Path;

use pest::Parser;
use pest::iterators::Pairs;
use symbol::Symbol;

use error::Error;
use literal::Literal;

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
    OftLispParser::parse(Rule::program, src).map(|pairs| convert_program(pairs))
}

fn convert_program(mut pairs: Pairs<Rule>) -> Vec<Literal> {
    let pairs = {
        let pair = pairs.next().unwrap();
        assert_eq!(pair.as_rule(), Rule::program);
        pair.into_inner()
    };

    let mut values = Vec::new();
    for pair in pairs {
        assert_eq!(pair.as_rule(), Rule::value);
        values.push(convert_value(pair.into_inner()));
    }
    values
}

fn convert_value(mut pairs: Pairs<Rule>) -> Literal {
    let pair = pairs.next().unwrap();
    match pair.as_rule() {
        Rule::bytes => unimplemented!(),
        Rule::list => convert_list(pair.into_inner()),
        Rule::rmacro => unimplemented!(),
        Rule::string => convert_string(pair.into_inner()),
        Rule::symbolish => Literal::Symbol(Symbol::from(pair.as_str())),
        Rule::vector => unimplemented!(),
        r => panic!("Invalid rule: {:?}", r),
    }
}

fn convert_list(pairs: Pairs<Rule>) -> Literal {
    let mut head = Literal::Nil;
    // TODO: https://github.com/pest-parser/pest/issues/205
    let pairs = pairs.collect::<Vec<_>>().into_iter().rev();
    for pair in pairs {
        assert_eq!(pair.as_rule(), Rule::value);
        let val = Box::new(convert_value(pair.into_inner()));
        head = Literal::Cons(val, Box::new(head));
    }
    head
}

//hex_esc = { "x" ~ hex_digit{2} }
//hex_digit = { '0'..'9' | 'a'..'f' | 'A'..'F' }
//predef_esc = { "n" | "r" | "t" | "\\" | "\"" | "'" }

//bytes = ${ "b\"" ~ (bytes_raw_ch | bytes_esc_ch)* ~ "\"" }
//bytes_raw_ch = { !("\\" | "\"") ~ ' '..'~' }
//bytes_esc_ch = { "\\" ~ (hex_esc | predef_esc) }

//list = { "(" ~ value* ~ ("|" ~ value)? ~ ")" }
//vector = { "[" ~ value* ~ "]" }

//rmacro = { rmacro_ch ~ value }
//rmacro_ch = { "'" | "`" | ",@" | "," | "\\" | "!" | "%" }

fn convert_string(pairs: Pairs<Rule>) -> Literal {
    let mut s = String::new();
    for pair in pairs {
        match pair.as_rule() {
            Rule::string_raw_ch => {
                s += pair.as_str();
            }
            Rule::string_4_esc => unimplemented!("{}", pair.as_str()),
            Rule::string_8_esc => unimplemented!("{}", pair.as_str()),
            Rule::string_esc_ch => unimplemented!("{}", pair.as_str()),
            r => panic!("Invalid rule: {:?}", r),
        }
    }
    //string = ${ "\"" ~ (string_raw_ch | string_esc_ch)* ~ "\"" }
    //string_raw_ch = { !("\\" | "\"") ~ any }
    //string_4_esc = { "u" ~ hex_digit{4} }
    //string_8_esc = { "U" ~ hex_digit{8} }
    //string_esc_ch = { "\\" ~ (hex_esc | string_4_esc | string_8_esc | predef_esc) }
    Literal::String(s)
}
