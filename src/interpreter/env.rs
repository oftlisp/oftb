use std::rc::Rc;

use interpreter::Value;

/// The (local) environment stack.
#[derive(Clone, Debug)]
pub struct Env {
    inner: Rc<EnvInner>,
}

#[derive(Clone, Debug)]
enum EnvInner {
    Cons(Value, Rc<EnvInner>),
    Nil,
}

impl Env {
    /// Creates a new, empty environment.
    pub fn new() -> Env {
        Env {
            inner: Rc::new(EnvInner::Nil),
        }
    }

    /// Gets a local variable.
    pub fn local(&self, mut depth: usize) -> Value {
        let mut e = &*self.inner;
        while let EnvInner::Cons(h, ref t) = *e {
            if depth == 0 {
                return h;
            }
            depth -= 1;
            e = &*t;
        }
        panic!("Environment stack underflow");
    }

    /// Adds a local variable.
    pub fn push(self, val: Value) -> Env {
        Env {
            inner: Rc::new(EnvInner::Cons(val, self.inner)),
        }
    }
}
