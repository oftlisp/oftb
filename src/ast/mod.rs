//! The types for the initial AST.

mod helpers;

use std::collections::BTreeSet;
use std::fmt::{Display, Formatter, Result as FmtResult};

use symbol::Symbol;

use heap::{Heap, HeapCell};

/// An error converting values to an AST.
#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Invalid declaration: {}", _0)] InvalidDecl(Literal),
    #[fail(display = "Invalid expression: {}", _0)] InvalidExpr(Literal),
    #[fail(display = "No module form was found.")] NoModuleForm,
}

/// A module.
#[derive(Clone, Debug, PartialEq)]
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
            helpers::convert_moduleish("module".into(), &l.remove(0))
                .ok_or_else(|| unimplemented!())?;
        let imports = {
            let i = l.iter()
                .position(|l| !helpers::is_moduleish("import".into(), l))
                .unwrap_or(0);
            l.drain(0..i)
                .map(|l| {
                    helpers::convert_moduleish("import".into(), &l).unwrap()
                })
                .flat_map(|(m, vs)| vs.into_iter().map(move |v| (m, v)))
                .collect()
        };
        let body = l.into_iter()
            .map(Decl::from_value)
            .collect::<Result<_, _>>()?;
        Ok(Module {
            name,
            imports,
            exports: exports.into_iter().collect(),
            body,
        })
    }
}

/// A declaration.
#[derive(Clone, Debug, PartialEq)]
pub enum Decl {
    Def(Symbol, Box<Expr>),
    Defn(Symbol, Vec<Symbol>, Vec<Expr>, Box<Expr>),
}

impl Decl {
    /// Creates a declaration from a literal.
    pub fn from_value(lit: Literal) -> Result<Decl, Error> {
        if helpers::is_shl("def".into(), &lit) {
            let mut l = helpers::as_list(&lit).unwrap();
            if l.len() != 3 {
                return Err(Error::InvalidDecl(lit));
            }
            let expr = Expr::from_value(l.pop().unwrap())?;
            let name = l.pop().unwrap();
            let name = unimplemented!("Decl::from_value {} {:?}", name, expr);
            Ok(Decl::Def(name, Box::new(expr)))
        } else if helpers::is_shl("defn".into(), &lit) {
            let mut l = helpers::as_list(&lit).unwrap();
            if l.len() < 4 {
                return Err(Error::InvalidDecl(lit));
            }
            let tail = Expr::from_value(l.pop().unwrap())?;
            let body = l.drain(3..)
                .map(Expr::from_value)
                .collect::<Result<_, _>>()?;
            let args = l.pop().unwrap();
            let name = l.pop().unwrap();
            let (name, args) =
                unimplemented!("Decl::from_value {} {}", name, args);
            Ok(Decl::Defn(name, args, body, Box::new(tail)))
        } else {
            Err(Error::InvalidDecl(lit))
        }
    }
}

/// The basic expression type.
#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Call(Box<Expr>, Vec<Expr>),
    Decl(Decl),
    Literal(Literal),
    Var(Symbol),
    // TODO
}

impl Expr {
    /// Creates an expression from a literal.
    pub fn from_value(l: Literal) -> Result<Expr, Error> {
        unimplemented!("Expr::from_value {}", l)
    }
}

/// Literal values.
#[derive(Clone, Debug, PartialEq)]
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
