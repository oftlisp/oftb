use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

use symbol::Symbol;

use interpreter::{Addr, Bytes, Closure, Kont, State, Store, Vector};
use util::{escape_bytes, escape_str};

/// The type of an intrinsic function.
#[derive(Copy, Clone)]
pub struct Intrinsic(
    pub 
        for<'program> fn(Vec<Value>, &mut Store<'program>, Vec<Kont<'program>>)
            -> State<'program>,
);

impl Debug for Intrinsic {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        write!(fmt, "{:p}", self.0 as *const ())
    }
}

impl Display for Intrinsic {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        // TODO: Try to get the name out of the symbol table or something.
        write!(fmt, "{:?}", self)
    }
}

impl PartialEq for Intrinsic {
    fn eq(&self, other: &Intrinsic) -> bool {
        (self.0 as *const ()) == (other.0 as *const ())
    }
}

/// A value stored in the store.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Value {
    Byte(u8),
    Bytes(Addr<Bytes>, usize),
    Closure(Addr<Closure>),
    Cons(Addr<Value>, Addr<Value>),
    Fixnum(isize),
    Intrinsic(Intrinsic),
    Nil,
    String(Addr<String>, usize),
    Symbol(Symbol),
    Vector(Addr<Vector>, usize),
}

impl Value {
    /// Returns an object that impls `Display`, which will display this value.
    pub fn display<'store, 'program: 'store>(
        self,
        store: &'store Store<'program>,
    ) -> DisplayValue<'store, 'program> {
        DisplayValue { value: self, store }
    }

    /// Determines if two values are "deeply" equal. Note that this may take
    /// arbitrarily long, or even not terminate.
    pub fn equals(self, other: Value, store: &Store) -> bool {
        match (self, other) {
            (Value::Byte(l), Value::Byte(r)) => l == r,
            (Value::Bytes(la, ln), Value::Bytes(ra, rn)) => if ln == rn {
                store.get_bytes(la, ln) == store.get_bytes(ra, rn)
            } else {
                false
            },
            (Value::Cons(lh, lt), Value::Cons(rh, rt)) => {
                let lh = store.get(lh);
                let lt = store.get(lt);
                let rh = store.get(rh);
                let rt = store.get(rt);
                lh.equals(rh, store) && lt.equals(rt, store)
            }
            (Value::Fixnum(l), Value::Fixnum(r)) => l == r,
            (Value::Intrinsic(l), Value::Intrinsic(r)) => l == r,
            (Value::Nil, Value::Nil) => true,
            (Value::String(la, ln), Value::String(ra, rn)) => if ln == rn {
                store.get_str(la, ln) == store.get_str(ra, rn)
            } else {
                false
            },

            (Value::Symbol(l), Value::Symbol(r)) => l == r,
            (Value::Vector(la, ln), Value::Vector(ra, rn)) => if ln == rn {
                let l = store.get_vec(la, ln);
                let r = store.get_vec(ra, rn);
                for i in 0..ln {
                    if !l[i].equals(r[i], store) {
                        return false;
                    }
                }
                true
            } else {
                false
            },
            _ => false,
        }
    }
}

/// Impls `Display` for `Value`.
pub struct DisplayValue<'store, 'program: 'store> {
    value: Value,
    store: &'store Store<'program>,
}

impl<'store, 'program: 'store> Display for DisplayValue<'store, 'program> {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        match self.value {
            Value::Byte(n) => write!(fmt, "{}", n),
            Value::Bytes(a, l) => escape_bytes(self.store.get_bytes(a, l), fmt),
            Value::Closure(_) => write!(fmt, "<<function>>"),
            Value::Cons(h, t) => {
                write!(fmt, "({}", self.store.get(h).display(self.store))?;
                let mut l = self.store.get(t);
                loop {
                    match l {
                        Value::Cons(h, t) => {
                            let h = self.store.get(h).display(self.store);
                            write!(fmt, " {}", h)?;
                            l = self.store.get(t);
                        }
                        Value::Nil => break,
                        _ => {
                            write!(fmt, " | {}", l.display(self.store))?;
                            break;
                        }
                    }
                }
                write!(fmt, ")")
            }
            Value::Fixnum(n) => write!(fmt, "{}", n),
            Value::Intrinsic(i) => write!(fmt, "<<function {}>>", i),
            Value::Nil => write!(fmt, "()"),
            Value::String(a, l) => escape_str(self.store.get_str(a, l), fmt),
            Value::Symbol(s) => write!(fmt, "{}", s),
            Value::Vector(a, l) => {
                write!(fmt, "[")?;
                let mut first = true;
                for v in self.store.get_vec(a, l) {
                    if first {
                        first = false;
                    } else {
                        write!(fmt, " ")?;
                    }
                    write!(fmt, "{}", v.display(self.store))?;
                }
                write!(fmt, "]")
            }
        }
    }
}
