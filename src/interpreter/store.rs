use symbol::Symbol;

use flatanf::Expr;

/// The type of the store.
#[derive(Debug)]
pub struct Store<'program> {
    bytes: Vec<u8>,
    strs: Vec<String>,
    vecs: Vec<Addr>,

    vals: Vec<Value<'program>>,
}

impl<'program> Store<'program> {
    /// Creates a new, empty heap.
    pub fn new() -> Store<'program> {
        Store {
            bytes: Vec::new(),
            strs: Vec::new(),
            vecs: Vec::new(),
            vals: vec![Value::Nil],
        }
    }

    /// Gets a value out of the value heap.
    pub fn get(&self, addr: Addr) -> Value {
        self.vals[addr.n]
    }

    /// Gets a bytes value out of the bytes heap.
    pub fn get_bytes(&self, addr: Addr, len: usize) -> &[u8] {
        let start = addr.n;
        let end = start + len;
        &self.bytes[start..end]
    }

    /// Gets a string out of the string heap.
    pub fn get_str(&self, addr: Addr) -> &str {
        &self.strs[addr.n]
    }

    /// Gets a vector out of the vector heap.
    pub fn get_vec(&self, addr: Addr, len: usize) -> Vec<Value> {
        let start = addr.n;
        let end = start + len;
        self.vecs[start..end]
            .iter()
            .map(|addr| self.vals[addr.n])
            .collect()
    }

    /// Stores a value into the value heap.
    pub fn store(&mut self, value: Value<'program>) -> Addr {
        let n = if let Value::Nil = value {
            0
        } else {
            let len = self.vals.len();
            self.vals.push(value);
            len
        };
        Addr { n }
    }
}

/// The address of a value in the store.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Addr {
    n: usize,
}

/// A value stored in the store.
#[derive(Clone, Copy, Debug)]
pub enum Value<'program> {
    Byte(u8),
    Bytes(Addr, usize),
    Closure(usize, &'program Expr),
    Cons(Addr, Addr),
    Fixnum(usize),
    Nil,
    String(Addr, usize),
    Symbol(Symbol),
    Vector(Addr, usize),
}
