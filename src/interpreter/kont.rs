use flatanf::Expr;
use interpreter::env::Env;
use interpreter::store::Addr;
use interpreter::value::Value;

/// A continuation on the continuation stack.
#[derive(Debug)]
pub enum Kont<'program> {
    /// A continuation for let evaluation.
    Let(&'program Expr, Env),

    /// A continuation of a call to `make-vector`.
    MakeVector(usize, usize, Value, Vec<Addr<Value>>),

    /// A continuation for seq evaluation.
    Seq(&'program Expr, Env),
}
