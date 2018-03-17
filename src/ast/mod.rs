//! The types for the initial AST.

mod helpers;

use std::collections::BTreeSet;

use literal::Literal;
use symbol::Symbol;

/// An error converting values to an AST.
#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Invalid declaration: {}", _0)] InvalidDecl(Literal),
    #[fail(display = "Invalid expression: {}", _0)] InvalidExpr(Literal),
    #[fail(display = "No module form was found.")] NoModuleForm,
}

/// A module.
#[derive(Clone, Debug, PartialEq)]
pub struct Module {
    pub name: Symbol,
    pub exports: BTreeSet<Symbol>,
    pub imports: BTreeSet<(Symbol, Symbol)>,
    pub body: Vec<Decl>,
}

impl Module {
    /// Creates a module from literals.
    pub fn from_values(mut l: Vec<Literal>) -> Result<Module, Error> {
        if l.len() == 0 {
            return Err(Error::NoModuleForm);
        }

        let (name, exports) =
            helpers::convert_moduleish("module".into(), &l.remove(0))
                .ok_or_else(|| unimplemented!())?;
        let imports = {
            let i = l.iter()
                .position(|l| !helpers::is_moduleish("import".into(), l))
                .unwrap_or(0);
            l.drain(0..i)
                .map(|l| {
                    helpers::convert_moduleish("import".into(), &l).unwrap()
                })
                .flat_map(|(m, vs)| vs.into_iter().map(move |v| (m, v)))
                .collect()
        };
        let body = l.into_iter()
            .map(Decl::from_value)
            .collect::<Result<_, _>>()?;
        Ok(Module {
            name,
            imports,
            exports: exports.into_iter().collect(),
            body,
        })
    }
}

/// A declaration.
#[derive(Clone, Debug, PartialEq)]
pub enum Decl {
    Def(Symbol, Expr),
    Defn(Symbol, Vec<Symbol>, Vec<Expr>, Expr),
}

impl Decl {
    /// Creates a declaration from a literal.
    pub fn from_value(lit: Literal) -> Result<Decl, Error> {
        if lit.is_shl("def".into()) {
            let mut l = lit.as_list().unwrap();
            if l.len() != 3 {
                return Err(Error::InvalidDecl(lit));
            }
            let expr = Expr::from_value(l.pop().unwrap())?;
            let name = if let Literal::Symbol(name) = l.pop().unwrap() {
                name
            } else {
                return Err(Error::InvalidDecl(lit));
            };
            Ok(Decl::Def(name, expr))
        } else if lit.is_shl("defn".into()) {
            let mut l = lit.as_list().unwrap();
            if l.len() < 4 {
                return Err(Error::InvalidDecl(lit));
            }
            let tail = Expr::from_value(l.pop().unwrap())?;
            let body = l.drain(3..)
                .map(Expr::from_value)
                .collect::<Result<_, _>>()?;
            let args = if let Some(args) = l.pop().unwrap().as_symbol_list() {
                args
            } else {
                return Err(Error::InvalidDecl(lit));
            };
            let name = if let Literal::Symbol(name) = l.pop().unwrap() {
                name
            } else {
                return Err(Error::InvalidDecl(lit));
            };
            Ok(Decl::Defn(name, args, body, tail))
        } else {
            Err(Error::InvalidDecl(lit))
        }
    }
}

/// The basic expression type.
#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Call(Box<Expr>, Vec<Expr>),
    Decl(Box<Decl>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    Lambda(Vec<Symbol>, Vec<Expr>, Box<Expr>),
    Literal(Literal),
    Progn(Vec<Expr>, Box<Expr>),
    Var(Symbol),
    Vector(Vec<Expr>),
}

impl Expr {
    /// Creates an expression from a literal.
    pub fn from_value(lit: Literal) -> Result<Expr, Error> {
        match lit {
            Literal::Cons(h, t) => {
                let t = match t.as_list() {
                    Some(t) => t,
                    None => return Err(Error::InvalidExpr(Literal::Cons(h, t))),
                };
                match *h {
                    Literal::Symbol(s)
                        if s.as_str() == "def" || s.as_str() == "defn" =>
                    {
                        unimplemented!()
                    }
                    Literal::Symbol(s) if s.as_str() == "if" => {
                        unimplemented!()
                    }
                    Literal::Symbol(s) if s.as_str() == "quote" => {
                        unimplemented!()
                    }
                    Literal::Symbol(s) if s.as_str() == "progn" => {
                        unimplemented!()
                    }
                    _ => {
                        let func = Expr::from_value(*h)?;
                        let args = t.into_iter()
                            .map(Expr::from_value)
                            .collect::<Result<_, _>>()?;
                        Ok(Expr::Call(Box::new(func), args))
                    }
                }
            }
            Literal::Nil => Err(Error::InvalidExpr(Literal::Nil)),
            Literal::Symbol(s) => Ok(Expr::Var(s)),
            Literal::Vector(vs) => vs.into_iter()
                .map(Expr::from_value)
                .collect::<Result<_, _>>()
                .map(Expr::Vector),
            lit => Ok(Expr::Literal(lit)),
        }
    }
}
