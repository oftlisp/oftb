use std::cmp::Ordering;
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
    /// Returns an object that will display this value. If the `printlike`
    /// argument is true, strings will not be escaped.
    pub fn display<'store, 'program: 'store>(
        self,
        store: &'store Store<'program>,
        printlike: bool,
    ) -> impl 'store + Display {
        DisplayValue {
            value: self,
            printlike,
            store,
        }
    }

    /// Compares two values "deeply". Note that this may take arbitrarily long,
    /// or even not terminate (for cyclic structures).
    pub fn compare(self, other: Value, store: &Store) -> Ordering {
        match (self, other) {
            (Value::Byte(l), Value::Byte(r)) => l.cmp(&r),
            (Value::Byte(_), _) => Ordering::Less,

            (Value::Bytes(_, _), Value::Byte(_)) => Ordering::Greater,
            (Value::Bytes(la, ln), Value::Bytes(ra, rn)) => store
                .get_bytes(la, ln)
                .cmp(store.get_bytes(ra, rn)),
            (Value::Bytes(_, _), _) => Ordering::Less,

            (Value::Cons(_, _), Value::Byte(_)) => Ordering::Greater,
            (Value::Cons(_, _), Value::Bytes(_, _)) => Ordering::Greater,
            /*
            (Value::Cons(lh, lt), Value::Cons(rh, rt)) => {
                let lh = store.get(lh);
                let lt = store.get(lt);
                let rh = store.get(rh);
                let rt = store.get(rt);
                lh.equals(rh, store) && lt.equals(rt, store)
            }
            */
            (Value::Cons(_, _), _) => Ordering::Less,

            (Value::Fixnum(_), Value::Byte(_)) => Ordering::Greater,
            (Value::Fixnum(_), Value::Bytes(_, _)) => Ordering::Greater,
            (Value::Fixnum(_), Value::Cons(_, _)) => Ordering::Greater,
            (Value::Fixnum(l), Value::Fixnum(r)) => l.cmp(&r),
            (Value::Fixnum(_), _) => Ordering::Less,

            (Value::Intrinsic(_), Value::Byte(_)) => Ordering::Greater,
            (Value::Intrinsic(_), Value::Bytes(_, _)) => Ordering::Greater,
            (Value::Intrinsic(_), Value::Cons(_, _)) => Ordering::Greater,
            (Value::Intrinsic(_), Value::Fixnum(_)) => Ordering::Greater,
            (Value::Intrinsic(l), Value::Intrinsic(r)) => l.cmp(&r),
            (Value::Intrinsic(_), _) => Ordering::Less,

            (Value::Nil, Value::Byte(_)) => Ordering::Greater,
            (Value::Nil, Value::Bytes(_, _)) => Ordering::Greater,
            (Value::Nil, Value::Cons(_, _)) => Ordering::Greater,
            (Value::Nil, Value::Fixnum(_)) => Ordering::Greater,
            (Value::Nil, Value::Intrinsic(r)) => Ordering::Greater,
            (Value::Nil, Value::Nil) => Ordering::Equal,
            (Value::Nil, _) => Ordering::Less,

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
            */
        }
    }

    /// Determines if two values are "deeply" equal. Note that this may take
    /// arbitrarily long, or even not terminate (for cyclic structures).
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

struct DisplayValue<'store, 'program: 'store> {
    value: Value,
    printlike: bool,
    store: &'store Store<'program>,
}

impl<'store, 'program: 'store> Display for DisplayValue<'store, 'program> {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        match self.value {
            Value::Byte(n) => write!(fmt, "{}", n),
            Value::Bytes(a, l) => escape_bytes(self.store.get_bytes(a, l), fmt),
            Value::Closure(a) => {
                if let Some(name) = self.store.get_closure(a).2 {
                    write!(fmt, "<<function {}>>", name)
                } else {
                    write!(fmt, "<<function>>")
                }
            }
            Value::Cons(h, t) => {
                write!(
                    fmt,
                    "({}",
                    self.store.get(h).display(self.store, false)
                )?;
                let mut l = self.store.get(t);
                loop {
                    match l {
                        Value::Cons(h, t) => {
                            let h =
                                self.store.get(h).display(self.store, false);
                            write!(fmt, " {}", h)?;
                            l = self.store.get(t);
                        }
                        Value::Nil => break,
                        _ => {
                            write!(fmt, " | {}", l.display(self.store, false))?;
                            break;
                        }
                    }
                }
                write!(fmt, ")")
            }
            Value::Fixnum(n) => write!(fmt, "{}", n),
            Value::Intrinsic(i) => write!(fmt, "<<function {}>>", i),
            Value::Nil => write!(fmt, "()"),
            Value::String(a, l) => {
                if self.printlike {
                    write!(fmt, "{}", self.store.get_str(a, l))
                } else {
                    escape_str(self.store.get_str(a, l), fmt)
                }
            }
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
                    write!(fmt, "{}", v.display(self.store, false))?;
                }
                write!(fmt, "]")
            }
        }
    }
}
