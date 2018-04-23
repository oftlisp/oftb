//! A "flattened" version of the `anf` AST. Uses De Bruijn indices instead of
//! local variable names, and explicitly includes a fully-qualified path to all
//! global variables.

mod convert;
mod serialize;
mod util;

use symbol::Symbol;

use literal::Literal;

/// A complete program.
#[derive(Clone, Debug, PartialEq)]
pub struct Program(pub Vec<Decl>);

/// A declaration.
#[derive(Clone, Debug, PartialEq)]
pub enum Decl {
    Def(Symbol, Literal),
    Defn(Symbol, Vec<Symbol>, Expr),
}

impl Decl {
    /// Returns the name of the Decl.
    pub fn name(&self) -> Symbol {
        match *self {
            Decl::Def(name, _) => name,
            Decl::Defn(name, _, _) => name,
        }
    }
}

/// The root expression type, which may perform arbitrary continuation stack
/// manipulation.
#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    AExpr(AExpr),
    CExpr(CExpr),
    Let(Box<Expr>, Box<Expr>),
    Seq(Box<Expr>, Box<Expr>),
}

/// A "complex" expression, which may replace the current continuation and have
/// side effects, but may not push to or pop from the continuation stack.
#[derive(Clone, Debug, PartialEq)]
pub enum CExpr {
    Call(AExpr, Vec<AExpr>),
    If(AExpr, Box<Expr>, Box<Expr>),
    LetRec(Vec<AExpr>, Box<Expr>),
}

/// An atomic expression, which must immediately evaluate to a value without
/// side effects, popping from the continuation stack.
#[derive(Clone, Debug, PartialEq)]
pub enum AExpr {
    Global(Symbol),
    Lambda(usize, Box<Expr>),
    Literal(Literal),
    Local(usize),
    Vector(Vec<AExpr>),
}
