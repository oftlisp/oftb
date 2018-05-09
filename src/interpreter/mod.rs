//! Interpretation for the `flatanf` AST.

mod control;
mod env;
pub mod eval;
mod state;
mod store;
mod kont;

use std::collections::HashMap;

use symbol::Symbol;

use flatanf::Expr;
pub use interpreter::control::Control;
use interpreter::env::Env;
pub use interpreter::kont::Kont;
pub use interpreter::state::State;
pub use interpreter::store::{Addr, Store, Value};

/// The interpreter.
#[derive(Debug)]
pub struct Interpreter<'program> {
    /// The global environment.
    pub globals: HashMap<Symbol, Value>,

    /// The store.
    pub store: Store<'program>,

    /// The state of the interpreter. Uses an Option to allow moving out.
    state: Option<State<'program>>,
}

impl<'program> Interpreter<'program> {
    /// Creates a new interpreter with an empty set of globals and an empty
    /// store. Starts in the halt state.
    pub fn new() -> Interpreter<'program> {
        Interpreter::with_globals_and_store(HashMap::new(), Store::new())
    }

    /// Creates a new interpreter with the given globals and store. Starts
    /// in the halt state.
    pub fn with_globals_and_store(
        globals: HashMap<Symbol, Value>,
        store: Store<'program>,
    ) -> Interpreter<'program> {
        let state = Some(State::Halted(Value::Nil));
        Interpreter {
            store,
            globals,
            state,
        }
    }

    /// Evaluates an expression to a value.
    pub fn eval(&mut self, expr: &'program Expr) -> Value {
        self.load_expr(expr);
        loop {
            if let Some(value) = self.eval_step() {
                return value;
            }
        }
    }

    /// Makes an evaluation step, returning a value if evaluation halted.
    pub fn eval_step(&mut self) -> Option<Value> {
        let state = self.state.take().unwrap();
        let next = match state {
            State::Running(c, e, k) => {
                eval::step(c, e, k, &self.globals, &mut self.store)
            }
            State::Halted(val) => State::Halted(val),
        };
        self.state = Some(next);

        if let Some(State::Halted(val)) = self.state {
            Some(val)
        } else {
            None
        }
    }

    /// Loads an expression into the interpreter, erasing any previous
    /// evaluation state.
    pub fn load_expr(&mut self, expr: &'program Expr) {
        self.state = Some(State::Running(
            Control::Normal(expr),
            Env::new(),
            Vec::new(),
        ));
    }
}
