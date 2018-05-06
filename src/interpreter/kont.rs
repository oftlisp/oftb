use flatanf::Expr;

/// A continuation on the continuation stack.
#[derive(Debug)]
pub enum Kont<'program> {
    /// A continuation for let evaluation.
    Let(&'program Expr),

    /// A continuation for seq evaluation.
    Seq(&'program Expr),
}
