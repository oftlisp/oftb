//! Interpretation for the `flatanf` AST.

use flatanf::Program;

mod control;
mod env;
mod eval;
mod state;
mod store;
mod kont;

use self::state::State;
pub use self::store::{Addr, Store, Value};

/// The interpreter.
#[derive(Debug)]
pub struct Interpreter<'program> {
    /// The state of the interpreter. Uses an Option to allow moving out.
    state: Option<State<'program>>,
}

impl<'program> Interpreter<'program> {
    /// Makes an evaluation step, returning a value if evaluation halted.
    pub fn eval_step(&mut self) -> Option<Value> {
        let state = self.state.take().unwrap();
        self.state = Some(eval::step(state));
        if let Some(State::Halted(val)) = self.state {
            Some(val)
        } else {
            None
        }
    }
}
