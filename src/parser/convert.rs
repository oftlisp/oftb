use pest::Error;
use pest::iterators::{Pair, Pairs};
use symbol::Symbol;

use literal::Literal;
use parser::Rule;

pub fn convert_program(
    mut pairs: Pairs<Rule>,
) -> Result<Vec<Literal>, Error<Rule>> {
    let pair = pairs.next().unwrap();
    assert_eq!(pair.as_rule(), Rule::program);
    pair.into_inner()
        .map(|pair| {
            assert_eq!(pair.as_rule(), Rule::value);
            convert_value(pair.into_inner())
        })
        .collect()
}

fn convert_value(mut pairs: Pairs<Rule>) -> Result<Literal, Error<Rule>> {
    let pair = pairs.next().unwrap();
    match pair.as_rule() {
        Rule::bytes => convert_bytes(pair.into_inner()),
        Rule::list => convert_list(pair.into_inner()),
        Rule::rmacro => {
            let mut pairs = pair.into_inner();
            let rmacro = pairs.next().unwrap();
            let value = convert_value(pairs.next().unwrap().into_inner())?;
            assert!(pairs.next().is_none());
            convert_rmacro(rmacro, value)
        }
        Rule::string => convert_string(pair.into_inner()),
        Rule::symbolish => Ok(Literal::Symbol(Symbol::from(pair.as_str()))),
        Rule::vector => pair.into_inner()
            .map(|pair| {
                assert_eq!(pair.as_rule(), Rule::value);
                convert_value(pair.into_inner())
            })
            .collect::<Result<_, _>>()
            .map(Literal::Vector),
        r => panic!("Invalid rule: {:?}", r),
    }
}

fn convert_list(pairs: Pairs<Rule>) -> Result<Literal, Error<Rule>> {
    let mut head = Literal::Nil;
    // TODO: https://github.com/pest-parser/pest/issues/205
    let pairs = pairs.collect::<Vec<_>>().into_iter().rev();
    for pair in pairs {
        assert_eq!(pair.as_rule(), Rule::value);
        let val = Box::new(convert_value(pair.into_inner())?);
        head = Literal::Cons(val, Box::new(head));
    }
    Ok(head)
}

fn convert_bytes(pairs: Pairs<Rule>) -> Result<Literal, Error<Rule>> {
    let mut bs = Vec::new();
    for pair in pairs {
        match pair.as_rule() {
            Rule::bytes_raw_ch => {
                let s = pair.as_str();
                assert_eq!(s.len(), 1);
                let n = s.chars().next().unwrap() as u32;
                assert!(n <= 0xff);
                bs.push(n as u8);
            }
            Rule::bytes_esc_ch => {
                let pair = pair.into_inner().next().unwrap();
                bs.push(convert_byte_escape(pair));
            }
            r => panic!("Invalid rule: {:?}", r),
        }
    }
    Ok(Literal::Bytes(bs))
}

fn convert_rmacro(
    pair: Pair<Rule>,
    value: Literal,
) -> Result<Literal, Error<Rule>> {
    fn simple_macro(name: &'static str, value: Literal) -> Literal {
        Literal::Cons(
            Box::new(Literal::Symbol(name.into())),
            Box::new(Literal::Cons(Box::new(value), Box::new(Literal::Nil))),
        )
    }

    Ok(match pair.as_str() {
        "'" => simple_macro("quote", value),
        "`" => simple_macro("quasiquote", value),
        ",@" => simple_macro("unquote-splicing", value),
        "," => simple_macro("unquote", value),
        "\\" => Literal::Cons(
            Box::new(Literal::Symbol("fn".into())),
            Box::new(Literal::Cons(
                Box::new(Literal::Cons(
                    Box::new(Literal::Symbol("$".into())),
                    Box::new(Literal::Nil),
                )),
                Box::new(Literal::Cons(
                    Box::new(value),
                    Box::new(Literal::Nil),
                )),
            )),
        ),
        "%" => simple_macro("debug-trace", value),
        rm => panic!("Invalid reader macro: {:?}", rm),
    })
}

fn convert_string(pairs: Pairs<Rule>) -> Result<Literal, Error<Rule>> {
    let mut s = String::new();
    for pair in pairs {
        match pair.as_rule() {
            Rule::string_raw_ch => {
                s += pair.as_str();
            }
            Rule::string_esc_ch => {
                s.push(convert_string_escape(pair)?);
            }
            r => panic!("Invalid rule: {:?}", r),
        }
    }
    Ok(Literal::String(s))
}

fn convert_string_escape(old_pair: Pair<Rule>) -> Result<char, Error<Rule>> {
    let pair = old_pair.clone().into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::string_4_esc | Rule::string_8_esc => {
            let n = convert_hex_escape(pair.into_inner());
            ::std::char::from_u32(n).ok_or_else(move || {
                Error::CustomErrorSpan {
                    message: format!(
                        "Invalid Unicode Escape: {}",
                        old_pair.as_str()
                    ),
                    span: old_pair.into_span(),
                }
            })
        }
        _ => Ok(convert_byte_escape(pair) as char),
    }
}

fn convert_byte_escape(pair: Pair<Rule>) -> u8 {
    match pair.as_rule() {
        Rule::hex_esc => convert_hex_escape(pair.into_inner()) as u8,
        Rule::predef_esc => match pair.as_str() {
            "n" => b'\n',
            "r" => b'\r',
            "t" => b'\t',
            "\\" => b'\\',
            "\"" => b'"',
            "'" => b'\'',
            e => panic!("Invalid escape: '\\{}'", e),
        },
        r => panic!("Invalid rule: {:?}", r),
    }
}

fn convert_hex_escape(pairs: Pairs<Rule>) -> u32 {
    let mut n = 0;
    for pair in pairs {
        assert_eq!(pair.as_rule(), Rule::hex_digit);
        n = (n << 4) + match pair.as_str() {
            "0" => 0,
            "1" => 1,
            "2" => 2,
            "3" => 3,
            "4" => 4,
            "5" => 5,
            "6" => 6,
            "7" => 7,
            "8" => 8,
            "9" => 9,
            "a" | "A" => 10,
            "b" | "B" => 11,
            "c" | "C" => 12,
            "d" | "D" => 13,
            "e" | "E" => 14,
            "f" | "F" => 15,
            d => panic!("Invalid hex digit: {:?}", d),
        };
    }
    n
}
