use flatanf::Expr;

/// The control value.
#[derive(Debug)]
pub enum Control<'program> {
    Normal(&'program Expr),
}
