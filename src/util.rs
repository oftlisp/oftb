use std::fmt::{Formatter, Result as FmtResult};

/// Converts a string containing a single hex digit to that digit, panicking if
/// the string does not contain exactly one hex digit.
pub fn convert_hex_digit(s: &str) -> u8 {
    match s {
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
        _ => panic!("Invalid hex digit: {:?}", s),
    }
}

/// Writes an escaped bytes literal to the given Formatter.
pub fn escape_bytes(bs: &[u8], fmt: &mut Formatter) -> FmtResult {
    write!(fmt, "b\"")?;
    for b in bs {
        write!(fmt, "\\x{:02x}", b)?;
    }
    write!(fmt, "\"")
}

/// Writes an escaped string to the given Formatter.
pub fn escape_str(s: &str, fmt: &mut Formatter) -> FmtResult {
    write!(fmt, "\"")?;
    for ch in s.chars() {
        let n = ch as u32;
        match ch {
            '\n' => write!(fmt, "\\n")?,
            '\r' => write!(fmt, "\\r")?,
            '\t' => write!(fmt, "\\t")?,
            '\\' => write!(fmt, "\\\\")?,
            '\"' => write!(fmt, "\\\"")?,
            _ => if n >= 0x20 && n < 0x7f {
                write!(fmt, "{}", ch)?;
            } else if n <= 0xff {
                write!(fmt, "\\x{:02x}", n)?;
            } else if n <= 0xffff {
                write!(fmt, "\\u{:04x}", n)?;
            } else {
                write!(fmt, "\\U{:08x}", n)?;
            },
        }
    }
    write!(fmt, "\"")
}
