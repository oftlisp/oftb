//! The types for the initial AST.

mod helpers;

use std::collections::BTreeSet;

use error::{Error, ErrorKind};
use literal::Literal;
use symbol::Symbol;

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
            return Err(ErrorKind::NoModuleForm.into());
        }

        let (name, exports) =
            helpers::convert_moduleish("module".into(), &l.remove(0))
                .ok_or_else(|| ErrorKind::NoModuleForm)?;
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
                return Err(ErrorKind::InvalidDecl(lit).into());
            }
            let expr = Expr::from_value(l.pop().unwrap())?;
            let name = if let Literal::Symbol(name) = l.pop().unwrap() {
                name
            } else {
                return Err(ErrorKind::InvalidDecl(lit).into());
            };
            Ok(Decl::Def(name, expr))
        } else if lit.is_shl("defn".into()) {
            let mut l = lit.as_list().unwrap();
            if l.len() < 4 {
                return Err(ErrorKind::InvalidDecl(lit).into());
            }
            let tail = Expr::from_value(l.pop().unwrap())?;
            let body = l.drain(3..)
                .map(Expr::from_value)
                .collect::<Result<_, _>>()?;
            let args = if let Some(args) = l.pop().unwrap().as_symbol_list() {
                args
            } else {
                return Err(ErrorKind::InvalidDecl(lit).into());
            };
            let name = if let Literal::Symbol(name) = l.pop().unwrap() {
                name
            } else {
                return Err(ErrorKind::InvalidDecl(lit).into());
            };
            Ok(Decl::Defn(name, args, body, tail))
        } else {
            Err(ErrorKind::InvalidDecl(lit).into())
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
    /// Creates an expression representing the nil literal.
    pub fn nil() -> Expr {
        Expr::Literal(Literal::Nil)
    }

    /// Creates an expression from a literal.
    pub fn from_value(lit: Literal) -> Result<Expr, Error> {
        match lit {
            Literal::Cons(h, t_lit) => {
                let mut t = match t_lit.as_list() {
                    Some(t) => t,
                    None => {
                        return Err(ErrorKind::InvalidExpr(Literal::Cons(
                            h,
                            t_lit,
                        )).into())
                    }
                };
                match *h {
                    Literal::Symbol(s)
                        if s.as_str() == "def" || s.as_str() == "defn" =>
                    {
                        let lit = Literal::Cons(h, t_lit);
                        let decl = Decl::from_value(lit)?;
                        Ok(Expr::Decl(Box::new(decl)))
                    }
                    Literal::Symbol(s) if s.as_str() == "if" => {
                        if t.len() < 2 || t.len() > 3 {
                            return Err(ErrorKind::InvalidExpr(Literal::Cons(
                                h,
                                t_lit,
                            )).into());
                        }

                        let else_expr = if t.len() == 3 {
                            Expr::from_value(t.pop().unwrap())?
                        } else {
                            Expr::nil()
                        };

                        let then_expr = Expr::from_value(t.pop().unwrap())?;
                        let cond_expr = Expr::from_value(t.pop().unwrap())?;
                        Ok(Expr::If(
                            Box::new(cond_expr),
                            Box::new(then_expr),
                            Box::new(else_expr),
                        ))
                    }
                    Literal::Symbol(s) if s.as_str() == "quote" => {
                        if t.len() != 1 {
                            return Err(ErrorKind::InvalidExpr(Literal::Cons(
                                h,
                                t_lit,
                            )).into());
                        }

                        Ok(Expr::Literal(t.pop().unwrap()))
                    }
                    Literal::Symbol(s) if s.as_str() == "progn" => {
                        if t.is_empty() {
                            Ok(Expr::Progn(Vec::new(), Box::new(Expr::nil())))
                        } else {
                            let tail = Expr::from_value(t.pop().unwrap())?;
                            let body = t.into_iter()
                                .map(Expr::from_value)
                                .collect::<Result<_, _>>()?;
                            Ok(Expr::Progn(body, Box::new(tail)))
                        }
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
            Literal::Nil => Err(ErrorKind::InvalidExpr(Literal::Nil).into()),
            Literal::Symbol(s) => Ok(Expr::Var(s)),
            Literal::Vector(vs) => vs.into_iter()
                .map(Expr::from_value)
                .collect::<Result<_, _>>()
                .map(Expr::Vector),
            lit => Ok(Expr::Literal(lit)),
        }
    }
}
