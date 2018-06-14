//! A "flattened" version of the `anf` AST. Uses De Bruijn indices instead of
//! local variable names, and explicitly includes a fully-qualified path to all
//! global variables.

mod convert;
mod deserialize;
mod display;
mod global_vars;
mod serialize;
mod util;

use std::collections::HashSet;

use symbol::Symbol;

use literal::Literal;

/// A complete program.
#[derive(Clone, Debug, PartialEq)]
pub struct Program {
    /// The required intrinsics.
    pub intrinsics: HashSet<Symbol>,

    /// The declarations in the program.
    pub decls: Vec<(Symbol, Expr)>,
}

/// The root expression type, which may perform arbitrary continuation stack
/// manipulation.
#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    /// An atomic expression.
    AExpr(AExpr),

    /// A complex expression.
    CExpr(CExpr),

    /// A let-binding. Evaluates the left expression, adds it to the
    /// environment, then evaluates the right expression.
    Let(Box<Expr>, Box<Expr>),

    /// A sequencing. Evaluates the left expression, then the right expression.
    Seq(Box<Expr>, Box<Expr>),
}

/// A "complex" expression, which may replace the current continuation and have
/// side effects, but may not (net) push to or pop from the continuation stack.
#[derive(Clone, Debug, PartialEq)]
pub enum CExpr {
    /// A function call.
    Call(AExpr, Vec<AExpr>),

    /// A conditional.
    If(AExpr, Box<Expr>, Box<Expr>),

    /// A letrec, which only handles (mutually) recursive functions.
    LetRec(Vec<(usize, Expr)>, Box<Expr>),
}

/// An atomic expression, which must immediately evaluate to a value without
/// side effects, popping from the continuation stack.
#[derive(Clone, Debug, PartialEq)]
pub enum AExpr {
    /// A reference to a global value.
    Global(Symbol),

    /// A function abstraction.
    Lambda(usize, Box<Expr>),

    /// A literal value.
    Literal(Literal),

    /// A reference to a value in the environment.
    Local(usize),

    /// A vector creation.
    Vector(Vec<AExpr>),
}
