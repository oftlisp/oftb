use std::fs::File;
use std::io::Read;
use std::path::Path;

use pest::Parser;
use pest::iterators::Pairs;
use symbol::Symbol;

use error::Error;
use heap::{Heap, HeapCell};

#[derive(Parser)]
#[grammar = "oftlisp.pest"]
pub struct OftLispParser;

/// Parses an OftLisp program from a file. If successful, builds the program
/// tree as a list of values on the heap, and returns its address.
pub fn parse_file<P: AsRef<Path>>(
    path: P,
    heap: &mut Heap,
) -> Result<usize, Error> {
    let path = path.as_ref();
    let src = File::open(path)
        .and_then(|mut file| {
            let mut src = String::new();
            file.read_to_string(&mut src).map(|_| src)
        })
        .map_err(|err| {
            Error::CouldntOpenSource(path.display().to_string(), err)
        })?;
    parse_program(&src, heap).map_err(|err| {
        Error::Parse(path.display().to_string(), err.to_string())
    })
}

fn parse_program<'src>(
    src: &'src str,
    heap: &mut Heap,
) -> Result<usize, ::pest::Error<'src, Rule>> {
    OftLispParser::parse(Rule::program, src)
        .map(|pairs| convert_program(pairs, heap))
}

fn convert_program(mut pairs: Pairs<Rule>, heap: &mut Heap) -> usize {
    let pairs = {
        let pair = pairs.next().unwrap();
        assert_eq!(pair.as_rule(), Rule::program);
        pair.into_inner()
    };

    let mut values = Vec::new();
    for pair in pairs {
        assert_eq!(pair.as_rule(), Rule::value);
        values.push(convert_value(pair.into_inner(), heap));
    }
    heap.alloc_iter_as_list(values)
}

fn convert_value(mut pairs: Pairs<Rule>, heap: &mut Heap) -> HeapCell {
    let pair = pairs.next().unwrap();
    match pair.as_rule() {
        Rule::bytes => unimplemented!(),
        Rule::list => convert_list(pair.into_inner(), heap),
        Rule::rmacro => unimplemented!(),
        Rule::string => convert_string(pair.into_inner(), heap),
        Rule::symbolish => HeapCell::Symbol(Symbol::from(pair.as_str())),
        Rule::vector => unimplemented!(),
        r => panic!("Invalid rule: {:?}", r),
    }
}

fn convert_list(pairs: Pairs<Rule>, heap: &mut Heap) -> HeapCell {
    let mut head = HeapCell::Nil;
    // TODO: https://github.com/pest-parser/pest/issues/205
    let pairs = pairs.collect::<Vec<_>>().into_iter().rev();
    for pair in pairs {
        assert_eq!(pair.as_rule(), Rule::value);
        let val = convert_value(pair.into_inner(), heap);
        let val_addr = heap.alloc_cell(val);
        let list_addr = heap.alloc_cell(head);
        head = HeapCell::Cons(val_addr, list_addr);
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

fn convert_string(pairs: Pairs<Rule>, heap: &mut Heap) -> HeapCell {
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
    HeapCell::String(s)
}
