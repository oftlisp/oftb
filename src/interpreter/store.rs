use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::marker::PhantomData;

use Literal;
use flatanf::Expr;
use interpreter::{Env, Value};

/// A phantom type for `Addr<Bytes>`.
pub enum Bytes {}

/// A phantom type for `Addr<Closure>`.
pub enum Closure {}

/// A phantom type for `Addr<Vector>`.
pub enum Vector {}

/// The type of the store.
#[derive(Debug)]
pub struct Store<'program> {
    bytes: Vec<u8>,
    clos: Vec<(usize, &'program Expr, Env)>,
    strs: String,
    vecs: Vec<Addr<Value>>,

    vals: Vec<Value>,
}

impl<'program> Store<'program> {
    /// Creates a new, empty heap.
    pub fn new() -> Store<'program> {
        Store {
            bytes: Vec::new(),
            clos: Vec::new(),
            strs: String::new(),
            vecs: Vec::new(),
            vals: vec![Value::Nil],
        }
    }

    /// Gets a value out of the value heap.
    pub fn get(&self, addr: Addr<Value>) -> Value {
        self.vals[addr.0]
    }

    /// Gets a bytes value out of the bytes heap.
    pub fn get_bytes(&self, addr: Addr<Bytes>, len: usize) -> &[u8] {
        let start = addr.0;
        let end = start + len;
        &self.bytes[start..end]
    }

    /// Gets a closure out of the closure heap.
    pub fn get_closure(
        &self,
        addr: Addr<Closure>,
    ) -> (usize, &'program Expr, Env) {
        let &(addr, ref body, ref env) = &self.clos[addr.0];
        (addr, body, env.clone())
    }

    /// Gets a string out of the string heap.
    pub fn get_str(&self, addr: Addr<String>, len: usize) -> &str {
        let start = addr.0;
        let end = start + len;
        &self.strs[start..end]
    }

    /// Gets a vector out of the vector heap.
    pub fn get_vec(&self, addr: Addr<Vector>, len: usize) -> Vec<Value> {
        let start = addr.0;
        let end = start + len;
        self.vecs[start..end]
            .iter()
            .map(|addr| self.vals[addr.0])
            .collect()
    }

    /// Adds an item to the environment of the given closure. This is
    /// reasonably unsafe.
    pub fn mutate_closure_env(&mut self, addr: Addr<Closure>, env: Env) {
        self.clos[addr.0].2 = env;
    }

    /// Stores a value into the value heap.
    pub fn store(&mut self, value: Value) -> Addr<Value> {
        let n = if let Value::Nil = value {
            0
        } else {
            let len = self.vals.len();
            self.vals.push(value);
            len
        };
        Addr(n, PhantomData)
    }

    /// Stores a value into the bytes heap.
    pub fn store_bytes(&mut self, bs: &[u8]) -> (Addr<Bytes>, usize) {
        let n = self.bytes.len();
        self.bytes.extend(bs);
        debug_assert_eq!(self.bytes.len(), n + bs.len());
        (Addr(n, PhantomData), bs.len())
    }

    /// Stores a value into the closure heap.
    pub fn store_closure(
        &mut self,
        argn: usize,
        body: &'program Expr,
        env: Env,
    ) -> Addr<Closure> {
        let n = self.clos.len();
        self.clos.push((argn, body, env));
        Addr(n, PhantomData)
    }

    /// Builds a literal onto the value heap.
    pub fn store_literal(&mut self, lit: &Literal) -> Value {
        match *lit {
            Literal::Byte(n) => Value::Byte(n),
            Literal::Bytes(ref bs) => {
                let (a, l) = self.store_bytes(bs);
                Value::Bytes(a, l)
            }
            Literal::Cons(ref h, ref t) => {
                let h = self.store_literal(h);
                let t = self.store_literal(t);
                Value::Cons(self.store(h), self.store(t))
            }
            Literal::Fixnum(n) => Value::Fixnum(n),
            Literal::Nil => Value::Nil,
            Literal::String(ref s) => {
                let (a, l) = self.store_str(s);
                Value::String(a, l)
            }
            Literal::Symbol(s) => Value::Symbol(s),
            Literal::Vector(ref vs) => {
                let vs = vs.iter()
                    .map(|lit| {
                        let val = self.store_literal(lit);
                        self.store(val)
                    })
                    .collect::<Vec<_>>();
                let (a, l) = self.store_vec(&vs);
                Value::Vector(a, l)
            }
        }
    }

    /// Stores a string into the vector heap.
    pub fn store_str(&mut self, s: &str) -> (Addr<String>, usize) {
        let n = self.strs.len();
        self.strs += s;
        debug_assert_eq!(self.strs.len(), n + s.len());
        (Addr(n, PhantomData), s.len())
    }

    /// Stores a value into the vector heap.
    pub fn store_vec(
        &mut self,
        addrs: &[Addr<Value>],
    ) -> (Addr<Vector>, usize) {
        let n = self.vecs.len();
        self.vecs.extend(addrs);
        debug_assert_eq!(self.vecs.len(), n + addrs.len());
        (Addr(n, PhantomData), addrs.len())
    }
}

/// The address of a value in the store.
pub struct Addr<To>(usize, PhantomData<To>);

impl<T> Clone for Addr<T> {
    fn clone(&self) -> Addr<T> {
        Addr(self.0, PhantomData)
    }
}

impl<T> Copy for Addr<T> {}

impl<T> Debug for Addr<T> {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        write!(fmt, "{:p}", self.0 as *const ())
    }
}

impl<T> Eq for Addr<T> {}

impl<T> From<usize> for Addr<T> {
    fn from(n: usize) -> Addr<T> {
        Addr(n, PhantomData)
    }
}

impl<T> Into<usize> for Addr<T> {
    fn into(self) -> usize {
        self.0
    }
}

impl<T> PartialEq for Addr<T> {
    fn eq(&self, other: &Addr<T>) -> bool {
        self.0 == other.0
    }
}
