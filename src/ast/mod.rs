//! The types for the initial AST.

mod from_values;

use std::collections::BTreeSet;
use std::fmt::{Display, Formatter, Result as FmtResult};

use symbol::Symbol;

use heap::{Heap, HeapCell};

/// An error converting values to an AST.
#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "No module form was found.")] NoModuleForm,
}

/// A module.
#[derive(Debug)]
pub struct Module {
    pub name: Symbol,
    pub exports: BTreeSet<Symbol>,
    pub imports: BTreeSet<(Symbol, Symbol)>,
    pub body: Vec<Decl>,
}

impl Module {
    /// Creates a module from literals.
    pub fn from_values(mut l: Vec<Literal>) -> Result<Module, Error> {
        if l.len() == 0 {
            return Err(Error::NoModuleForm);
        }

        let (name, exports) =
            from_values::convert_moduleish("module".into(), &l.remove(0))
                .ok_or_else(|| unimplemented!())?;
        let imports = {
            let i = l.iter()
                .position(|l| !from_values::is_moduleish("import".into(), l))
                .unwrap_or(0);
            l.drain(0..i)
                .map(|l| {
                    from_values::convert_moduleish("import".into(), &l).unwrap()
                })
                .flat_map(|(m, vs)| vs.into_iter().map(move |v| (m, v)))
                .collect()
        };
        let body = l.into_iter().map(Decl::from_value).collect();
        Ok(Module {
            name,
            imports,
            exports: exports.into_iter().collect(),
            body,
        })
    }
}

/// A declaration.
#[derive(Debug)]
pub enum Decl {
    // TODO
}

impl Decl {
    /// Creates a declaration from a literal.
    pub fn from_value(l: Literal) -> Decl {
        unimplemented!()
    }
}

/// The basic expression type.
#[derive(Debug)]
pub enum Expr {
    Decl(Decl),
    Literal(Literal),
    // TODO
}

impl Expr {
    /// Creates an expression from a literal.
    pub fn from_value(l: Literal) -> Expr {
        unimplemented!()
    }
}

/// Literal values.
#[derive(Debug)]
pub enum Literal {
    Byte(u8),
    Bytes(Vec<u8>),
    Cons(Box<Literal>, Box<Literal>),
    Fixnum(usize),
    Nil,
    String(String),
    Symbol(Symbol),
    Vector(Vec<Literal>),
}

impl Literal {
    /// Builds a literal onto the heap, returning its address.
    pub fn build_onto(self, heap: &mut Heap) -> usize {
        let cell = match self {
            Literal::Byte(n) => HeapCell::Byte(n),
            Literal::Bytes(bs) => HeapCell::Bytes(bs),
            Literal::Cons(h, t) => {
                let h = h.build_onto(heap);
                let t = t.build_onto(heap);
                HeapCell::Cons(h, t)
            }
            Literal::Fixnum(n) => HeapCell::Fixnum(n),
            Literal::Nil => HeapCell::Nil,
            Literal::String(s) => HeapCell::String(s),
            Literal::Symbol(s) => HeapCell::Symbol(s),
            Literal::Vector(vs) => HeapCell::Vector(
                vs.into_iter().map(|v| v.build_onto(heap)).collect(),
            ),
        };
        heap.alloc_cell(cell)
    }
}

impl Display for Literal {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        match *self {
            Literal::Byte(n) => write!(fmt, "{}", n),
            Literal::Bytes(ref bs) => {
                write!(fmt, "b\"")?;
                for b in bs {
                    write!(fmt, "\\x{:02x}", b)?;
                }
                write!(fmt, "\"")
            }
            Literal::Cons(ref h, ref t) => {
                write!(fmt, "({}", h)?;
                let mut l = t;
                loop {
                    match **l {
                        Literal::Cons(ref h, ref t) => {
                            write!(fmt, " {}", h)?;
                            l = t;
                        }
                        Literal::Nil => break,
                        _ => {
                            write!(fmt, " | {}", l)?;
                            break;
                        }
                    }
                }
                write!(fmt, ")")
            }
            Literal::Fixnum(n) => write!(fmt, "{}", n),
            Literal::Nil => write!(fmt, "()"),
            Literal::String(ref s) => {
                write!(fmt, "\"")?;
                for ch in s.chars() {
                    match ch {
                        '\n' => write!(fmt, "\\n")?,
                        '\r' => write!(fmt, "\\r")?,
                        '\t' => write!(fmt, "\\t")?,
                        '\\' => write!(fmt, "\\\\")?,
                        '\"' => write!(fmt, "\\\"")?,
                        _ => write!(fmt, "{}", ch)?,
                    }
                }
                write!(fmt, "\"")
            }
            Literal::Symbol(s) => write!(fmt, "{}", s),
            Literal::Vector(ref vs) => {
                write!(fmt, "[")?;
                let mut first = true;
                for v in vs {
                    if first {
                        first = false;
                    } else {
                        write!(fmt, " ")?;
                    }
                    write!(fmt, "{}", v)?;
                }
                write!(fmt, "]")
            }
        }
    }
}
