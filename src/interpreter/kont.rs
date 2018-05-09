use flatanf::Expr;
use interpreter::env::Env;

/// A continuation on the continuation stack.
#[derive(Debug)]
pub enum Kont<'program> {
    /// A continuation for let evaluation.
    Let(&'program Expr, Env),

    /// A continuation for seq evaluation.
    Seq(&'program Expr, Env),
}
