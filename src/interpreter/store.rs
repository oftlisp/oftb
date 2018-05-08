use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::marker::PhantomData;

use symbol::Symbol;

use flatanf::Expr;
use interpreter::env::Env;

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
    strs: Vec<String>,
    vecs: Vec<Addr<Value>>,

    vals: Vec<Value>,
}

impl<'program> Store<'program> {
    /// Creates a new, empty heap.
    pub fn new() -> Store<'program> {
        Store {
            bytes: Vec::new(),
            clos: Vec::new(),
            strs: Vec::new(),
            vecs: Vec::new(),
            vals: vec![Value::Nil],
        }
    }

    /// Gets a value out of the value heap.
    pub fn get(&self, addr: Addr<Value>) -> Value {
        self.vals[addr.n]
    }

    /// Gets a bytes value out of the bytes heap.
    pub fn get_bytes(&self, addr: Addr<Bytes>, len: usize) -> &[u8] {
        let start = addr.n;
        let end = start + len;
        &self.bytes[start..end]
    }

    /// Gets a closure out of the closure heap.
    pub fn get_closure(
        &self,
        addr: Addr<Closure>,
    ) -> &(usize, &'program Expr, Env) {
        &self.clos[addr.n]
    }

    /// Gets a string out of the string heap.
    pub fn get_str(&self, addr: Addr<String>) -> &str {
        &self.strs[addr.n]
    }

    /// Gets a vector out of the vector heap.
    pub fn get_vec(&self, addr: Addr<Vector>, len: usize) -> Vec<Value> {
        let start = addr.n;
        let end = start + len;
        self.vecs[start..end]
            .iter()
            .map(|addr| self.vals[addr.n])
            .collect()
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
        Addr {
            n,
            _to: PhantomData,
        }
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
        Addr {
            n,
            _to: PhantomData,
        }
    }

    /// Stores a value into the vector heap.
    pub fn store_vec(&mut self, addrs: &[Addr<Value>]) -> Addr<Vector> {
        let n = self.vecs.len();
        self.vecs.extend(addrs);
        Addr {
            n,
            _to: PhantomData,
        }
    }
}

/// The address of a value in the store.
pub struct Addr<To> {
    n: usize,
    _to: PhantomData<To>,
}

impl<T> Clone for Addr<T> {
    fn clone(&self) -> Addr<T> {
        Addr {
            n: self.n,
            _to: PhantomData,
        }
    }
}

impl<T> Copy for Addr<T> {}

impl<T> Debug for Addr<T> {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        write!(fmt, "{:p}", self.n as *const ())
    }
}

impl<T> Eq for Addr<T> {}

impl<T> PartialEq for Addr<T> {
    fn eq(&self, other: &Addr<T>) -> bool {
        self.n == other.n
    }
}

/// A value stored in the store.
#[derive(Clone, Copy, Debug)]
pub enum Value {
    Byte(u8),
    Bytes(Addr<Bytes>, usize),
    Closure(Addr<Closure>),
    Cons(Addr<Value>, Addr<Value>),
    Fixnum(usize),
    Nil,
    String(Addr<String>, usize),
    Symbol(Symbol),
    Vector(Addr<Vector>, usize),
}
