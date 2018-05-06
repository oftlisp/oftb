//! A "flattened" version of the `anf` AST. Uses De Bruijn indices instead of
//! local variable names, and explicitly includes a fully-qualified path to all
//! global variables.

mod convert;
mod deserialize;
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
    /// A value definition.
    ///
    /// TODO: Literal should be an Expr?
    Def(Symbol, Literal),

    /// A function definition. The difference between a Def containing a lambda
    /// and a Defn is that the Defn gets itself as an implicit parameter.
    ///
    /// TODO: Actually implement the above.
    Defn(Symbol, usize, Expr),
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
/// side effects, but may not push to or pop from the continuation stack.
#[derive(Clone, Debug, PartialEq)]
pub enum CExpr {
    /// A function call.
    Call(AExpr, Vec<AExpr>),

    /// A conditional.
    If(AExpr, Box<Expr>, Box<Expr>),

    /// A letrec. Adds the bindings to the environment, then evaluates each,
    /// then evaluates the Expr.
    LetRec(Vec<AExpr>, Box<Expr>),
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

    /// A vector.
    ///
    /// TODO: Should this actually exist?
    Vector(Vec<AExpr>),
}
