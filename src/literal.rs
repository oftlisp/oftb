use std::fmt::{Display, Formatter, Result as FmtResult};

use symbol::Symbol;

use heap::{Heap, HeapCell};

/// A literal value.
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
    /// Returns the list this literal contains, or `None` if it does not contain
    /// one.
    pub fn as_list(&self) -> Option<Vec<Literal>> {
        let mut lit = self;
        let mut l = Vec::new();
        loop {
            match *lit {
                Literal::Cons(ref h, ref t) => {
                    l.push((**h).clone());
                    lit = &*t;
                }
                Literal::Nil => return Some(l),
                _ => return None,
            }
        }
    }

    /// Returns the list this literal contains, excluding the symbol which
    /// must be its head.
    pub fn as_shl(&self) -> Option<(Symbol, Vec<Literal>)> {
        if let Literal::Cons(ref h, ref t) = *self {
            if let Literal::Symbol(s) = **h {
                t.as_list().map(|l| (s, l))
            } else {
                None
            }
        } else {
            None
        }
    }

    /// If this literal is of the form `(SYMBOL VAL)`, returns `(SYMBOL, VAL)`.
    pub fn as_shp(&self) -> Option<(Symbol, Literal)> {
        if let Literal::Cons(ref h, ref t) = *self {
            if let Literal::Symbol(s) = **h {
                if let Literal::Cons(ref h, ref t) = **t {
                    if Literal::Nil == **t {
                        Some((s, (**h).clone()))
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Converts a list of symbols to a `Vec<Symbol>`.
    pub fn as_symbol_list(&self) -> Option<Vec<Symbol>> {
        let mut lit = self;
        let mut out = Vec::new();
        loop {
            match *lit {
                Literal::Cons(ref h, ref t) => {
                    if let Literal::Symbol(s) = **h {
                        out.push(s);
                        lit = &*t;
                    } else {
                        return None;
                    }
                }
                Literal::Nil => return Some(out),
                _ => return None,
            }
        }
    }

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

    /// Checks if a literal represents a proper list.
    pub fn is_list(&self) -> bool {
        let mut lit = self;
        loop {
            match *lit {
                Literal::Cons(_, ref t) => lit = &*t,
                Literal::Nil => return true,
                _ => return false,
            }
        }
    }

    /// Checks if a literal represents a list whose head is the given symbol.
    pub fn is_shl(&self, sym: Symbol) -> bool {
        if let Literal::Cons(ref h, ref t) = *self {
            Literal::Symbol(sym) == **h && t.is_list()
        } else {
            false
        }
    }

    /// Creates a list from a vector of literals.
    pub fn list(mut vec: Vec<Literal>) -> Literal {
        let mut head = Literal::Nil;
        while let Some(lit) = vec.pop() {
            head = Literal::Cons(Box::new(lit), Box::new(head));
        }
        head
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
