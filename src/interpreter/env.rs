use interpreter::store::Value;

/// The (local) environment stack.
#[derive(Clone, Debug)]
pub struct Env {
    stack: Vec<Value>,
}

impl Env {
    /// Creates a new, empty environment.
    pub fn new() -> Env {
        Env { stack: Vec::new() }
    }

    /// Gets a local variable.
    pub fn local(&self, depth: usize) -> Value {
        assert!(depth <= self.stack.len());
        self.stack[self.stack.len() - depth]
    }

    /// Adds a local variable.
    pub fn push(&mut self, val: Value) {
        self.stack.push(val);
    }
}
