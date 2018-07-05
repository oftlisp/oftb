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

/// Looks up an address in the symbol table, returning the name that corresponds to it if one could
/// be found.
#[cfg(feature = "elf")]
pub fn symname(addr: usize) -> Option<&'static str> {
    use libc::{c_int, c_void, dl_iterate_phdr, dl_phdr_info};
    use rustc_demangle::demangle;
    use std::cell::UnsafeCell;
    use std::fs::File;
    use std::io::Read;
    use std::mem::transmute;
    use xmas_elf::{
        sections::{SectionData, ShType}, symbol_table::Entry, ElfFile,
    };

    lazy_static! {
        static ref SYMTAB_VEC: Vec<(usize, Option<String>)> = {
            unsafe extern "C" fn callback(
                info: *mut dl_phdr_info,
                _size: usize,
                data: *mut c_void,
            ) -> c_int {
                let relo: &mut usize = transmute(data);
                if *(*info).dlpi_name == 0 {
                    *relo = (*info).dlpi_addr as usize;
                }
                0
            }

            let relocation = unsafe {
                let tmp = UnsafeCell::new(0);
                let ret = dl_iterate_phdr(Some(callback), tmp.get() as _);
                assert_eq!(ret, 0);
                tmp.into_inner()
            };

            File::open("/proc/self/exe")
                .and_then(|mut f| {
                    let mut buf = Vec::new();
                    f.read_to_end(&mut buf).map(|_| buf)
                })
                .ok()
                .and_then(|elf| {
                    let elf = ElfFile::new(&elf).ok().or_else(|| {
                        warn!("Couldn't load self as ELF file");
                        None
                    });
                    elf.as_ref().map(|elf| {
                        elf.section_iter()
                            .filter(|s| match s.get_type() {
                                Ok(ShType::SymTab) => true,
                                Ok(ShType::DynSym) => true,
                                _ => false,
                            })
                            .flat_map(|s| match s.get_data(elf) {
                                Ok(SectionData::DynSymbolTable32(syms)) => {
                                    syms.iter().map(|e| e as _).collect()
                                }
                                Ok(SectionData::DynSymbolTable64(syms)) => {
                                    syms.iter().map(|e| e as _).collect()
                                }
                                Ok(SectionData::SymbolTable32(syms)) => {
                                    syms.iter().map(|e| e as _).collect()
                                }
                                Ok(SectionData::SymbolTable64(syms)) => {
                                    syms.iter().map(|e| e as _).collect()
                                }
                                _ => Vec::new(),
                            })
                            .map(|e: &Entry| {
                                let addr = e.value() as usize + relocation;
                                let name =
                                    e.get_name(elf).ok().map(|name| demangle(name).to_string());
                                (addr, name)
                            })
                            .collect()
                    })
                })
                .unwrap_or_else(Vec::new)
        };
        static ref SYMTAB: &'static [(usize, Option<String>)] = { &*SYMTAB_VEC };
    }

    SYMTAB
        .iter()
        .filter(|&(ref a, _)| addr == *a)
        .filter_map(|&(_, ref name)| name.as_ref().map(|s| s as _))
        .next()
}
