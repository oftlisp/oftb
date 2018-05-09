use std::fmt::{Formatter, Result as FmtResult};

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
