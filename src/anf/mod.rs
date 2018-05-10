//! An AST using [A-Normal Form](https://en.wikipedia.org/wiki/A-normal_form).

mod convert;

use std::collections::BTreeSet;

use symbol::Symbol;

use ast::Attr;
use literal::Literal;

/// A module.
#[derive(Clone, Debug, PartialEq)]
pub struct Module {
    pub name: Symbol,
    pub exports: BTreeSet<Symbol>,
    pub imports: BTreeSet<(Symbol, Symbol)>,
    pub body: Vec<Decl>,
    pub attrs: Vec<Attr>,
}

/// A declaration.
#[derive(Clone, Debug, PartialEq)]
pub enum Decl {
    Def(Symbol, Expr),
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
    Let(Symbol, Box<Expr>, Box<Expr>),
    Seq(Box<Expr>, Box<Expr>),
}

/// A "complex" expression, which may replace the current continuation and have
/// side effects, but may not push to or pop from the continuation stack.
#[derive(Clone, Debug, PartialEq)]
pub enum CExpr {
    Call(AExpr, Vec<AExpr>),
    If(AExpr, Box<Expr>, Box<Expr>),
    LetRec(Vec<(Symbol, AExpr)>, Box<Expr>),
}

/// An atomic expression, which must immediately evaluate to a value without
/// side effects, popping from the continuation stack.
#[derive(Clone, Debug, PartialEq)]
pub enum AExpr {
    Lambda(Vec<Symbol>, Box<Expr>),
    Literal(Literal),
    Var(Symbol),
    Vector(Vec<AExpr>),
}
