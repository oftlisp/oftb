use num::{BigInt, BigRational, BigUint, ToPrimitive, Zero};
use pest::Parser;
use pest::iterators::{Pair, Pairs};

use literal::Literal;
use util::convert_hex_digit;

#[cfg(debug_assertions)]
const _GRAMMAR: &'static str = include_str!("../symbolish.pest");

#[derive(Parser)]
#[grammar = "symbolish.pest"]
pub struct SymbolishParser;

enum Number {
    Byte(BigUint),
    Fixnum(BigInt),
    Float(BigRational),
}

/// Converts a symbolish string into the appropriate literal.
pub fn parse_symbolish(s: &str) -> Result<Literal, String> {
    match SymbolishParser::parse(Rule::root, s) {
        Ok(pairs) => match convert(pairs) {
            Number::Byte(n) => unimplemented!(),
            Number::Fixnum(n) => if let Some(n) = n.to_isize() {
                Ok(Literal::Fixnum(n))
            } else {
                Err(format!("Invalid fixnum: {}", n))
            },
            Number::Float(n) => unimplemented!(),
        },
        Err(_) => Ok(Literal::Symbol(s.into())),
    }
}

fn convert(mut pairs: Pairs<Rule>) -> Number {
    let pair = pairs.next().unwrap();
    assert_eq!(pairs.next(), None);
    assert_eq!(pair.as_rule(), Rule::root);

    let mut pairs = pair.into_inner();
    let pair = pairs.next().unwrap();
    assert_eq!(pairs.next(), None);
    assert_eq!(pair.as_rule(), Rule::symbolish);

    convert_symbolish(pair.into_inner())
}

fn convert_symbolish(mut pairs: Pairs<Rule>) -> Number {
    let pair = pairs.next().unwrap();
    assert_eq!(pairs.next(), None);
    match pair.as_rule() {
        //Rule::byte => unimplemented!("{:#?}", pair),
        Rule::fixnum => Number::Fixnum(convert_fixnum(pair.into_inner())),
        Rule::float => unimplemented!("{:#?}", pair),
        r => panic!("Invalid rule: {:?}", r),
    }
}

fn convert_fixnum(mut pairs: Pairs<Rule>) -> BigInt {
    let sign = pairs.next().unwrap();
    let unsigned = pairs.next().unwrap();
    assert_eq!(pairs.next(), None);
    assert_eq!(sign.as_rule(), Rule::sign);
    assert_eq!(unsigned.as_rule(), Rule::unsigned);

    let sign = convert_sign(sign);
    let unsigned = convert_unsigned(unsigned.into_inner());

    let mut signed = BigInt::from(unsigned);
    if sign {
        -signed
    } else {
        signed
    }
}

fn convert_float(mut pairs: Pairs<Rule>) -> BigRational {
    unimplemented!("{:?}", pairs)
}

fn convert_sign(pair: Pair<Rule>) -> bool {
    match pair.as_str() {
        "" => false,
        "-" => true,
        s => panic!("Invalid sign: {:?}", s),
    }
}

fn convert_unsigned(mut pairs: Pairs<Rule>) -> BigUint {
    let pair = pairs.next().unwrap();
    assert_eq!(pairs.next(), None);
    match pair.as_rule() {
        Rule::bin_num => convert_bin_num(pair.into_inner()),
        Rule::dec_num => convert_dec_num(pair.into_inner()),
        Rule::hex_num => convert_hex_num(pair.into_inner()),
        r => panic!("Invalid rule: {:?}", r),
    }
}

fn convert_bin_num(mut pairs: Pairs<Rule>) -> BigUint {
    let mut n = BigUint::zero();
    for pair in pairs {
        assert_eq!(pair.as_rule(), Rule::bin_digit);
        n <<= 1;
        n += match pair.as_str() {
            "0" => 0u8,
            "1" => 1u8,
            s => panic!("Invalid binary digit: {:?}", s),
        };
    }
    n
}

fn convert_dec_num(mut pairs: Pairs<Rule>) -> BigUint {
    let mut n = BigUint::zero();
    for pair in pairs {
        assert_eq!(pair.as_rule(), Rule::dec_digit);
        n *= 10u8;
        n += match pair.as_str() {
            "0" => 0u8,
            "1" => 1u8,
            "2" => 2u8,
            "3" => 3u8,
            "4" => 4u8,
            "5" => 5u8,
            "6" => 6u8,
            "7" => 7u8,
            "8" => 8u8,
            "9" => 9u8,
            s => panic!("Invalid decimal digit: {:?}", s),
        };
    }
    n
}

fn convert_hex_num(mut pairs: Pairs<Rule>) -> BigUint {
    let mut n = BigUint::zero();
    for pair in pairs {
        assert_eq!(pair.as_rule(), Rule::hex_digit);
        n <<= 4;
        n += convert_hex_digit(pair.as_str());
    }
    n
}
