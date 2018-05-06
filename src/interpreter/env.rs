use std::collections::HashMap;

use symbol::Symbol;

use interpreter::store::Value;

/// The global stack and environment stack.
#[derive(Debug)]
pub struct Env<'program> {
    global: HashMap<Symbol, Value<'program>>,
    local: Vec<Value<'program>>,
}

impl<'program> Env<'program> {
    /// Gets a global variable.
    pub fn global(&self, name: Symbol) -> Value<'program> {
        self.global[&name]
    }

    /// Gets a local variable.
    pub fn local(&self, depth: usize) -> Value<'program> {
        assert!(depth <= self.local.len());
        self.local[self.local.len() - depth]
    }

    /// Adds a local variable.
    pub fn push(&mut self, val: Value<'program>) {
        self.local.push(val);
    }
}
