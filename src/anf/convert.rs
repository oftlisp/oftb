use symbol::Symbol;

use anf::{AExpr, CExpr, Decl, Expr, Module};
use ast::{Decl as AstDecl, Expr as AstExpr, Module as AstModule};
use gensym::gensym;
use literal::Literal;

impl From<AstModule> for Module {
    fn from(m: AstModule) -> Module {
        Module {
            name: m.name,
            exports: m.exports,
            imports: m.imports,
            body: m.body.into_iter().map(Decl::from).collect(),
        }
    }
}

impl From<AstDecl> for Decl {
    fn from(decl: AstDecl) -> Decl {
        match decl {
            AstDecl::Def(name, AstExpr::Literal(lit)) => Decl::Def(name, lit),
            AstDecl::Defn(name, args, body, tail) => {
                Decl::Defn(name, args, convert_block(body, *tail))
            }
        }
    }
}

impl From<AstExpr> for Expr {
    fn from(expr: AstExpr) -> Expr {
        match expr {
            AstExpr::Call(func, args) => {
                let mut context = Vec::new();
                let func = into_aexpr(*func, &mut context);
                let args = args.into_iter()
                    .map(|e| into_aexpr(e, &mut context))
                    .collect();
                apply_context(
                    Expr::CExpr(CExpr::Call(Box::new(func), args)),
                    context,
                )
            }
            AstExpr::Decl(decl) => match decl {
                // A def in the tail position can have no effect besides that
                // of evaluating its expr.
                AstDecl::Def(name, expr) => Expr::Seq(
                    Box::new((*expr).into()),
                    Box::new(Expr::AExpr(AExpr::Literal(Literal::Nil))),
                ),

                // A defn in the tail position can have no effect.
                AstDecl::Defn(_, _, _, _) => {
                    Expr::AExpr(AExpr::Literal(Literal::Nil))
                }
            },
            AstExpr::If(c, t, e) => {
                let mut context = Vec::new();
                let c = into_aexpr(*c, &mut context);
                apply_context(
                    Expr::CExpr(CExpr::If(
                        Box::new(c),
                        Box::new((*t).into()),
                        Box::new((*e).into()),
                    )),
                    context,
                )
            }
            AstExpr::Literal(lit) => Expr::AExpr(AExpr::Literal(lit)),
            AstExpr::Progn(body, tail) => convert_block(body, *tail),
            AstExpr::Var(n) => Expr::AExpr(AExpr::Var(n)),
        }
    }
}

/// Applies bindings specified in a context to the expression.
fn apply_context(mut expr: Expr, mut context: Vec<(Symbol, Expr)>) -> Expr {
    while let Some((name, bound)) = context.pop() {
        expr = Expr::Let(name, Box::new(bound), Box::new(expr));
    }
    expr
}

/// Attempts to convert an `ast::Expr` to an `anf::AExpr`, returning the
/// `ast::Expr` back if it's not possible.
fn convert_aexpr(expr: AstExpr) -> Result<AExpr, AstExpr> {
    match expr {
        // TODO: AExpr::Lambda(args, body),
        AstExpr::Literal(l) => Ok(AExpr::Literal(l)),
        AstExpr::Var(n) => Ok(AExpr::Var(n)),
        expr => Err(expr),
    }
}

/// Converts a lexical block into an ANF expression.
fn convert_block(body: Vec<AstExpr>, tail: AstExpr) -> Expr {
    let mut anf = tail.into();
    let mut lambdas = Vec::new();
    for expr in body.into_iter().rev() {
        match expr {
            AstExpr::Decl(AstDecl::Def(name, expr)) => {
                anf = Expr::Let(name, Box::new((*expr).into()), Box::new(anf));
            }
            AstExpr::Decl(AstDecl::Defn(name, args, body, tail)) => {
                lambdas.push((
                    name,
                    AExpr::Lambda(args, Box::new(convert_block(body, *tail))),
                ));
            }
            expr => {
                if !lambdas.is_empty() {
                    anf = Expr::CExpr(CExpr::LetRec(lambdas, Box::new(anf)));
                    lambdas = Vec::new();
                }
                anf = Expr::Seq(Box::new(expr.into()), Box::new(anf));
            }
        };
    }
    anf
}

/// Converts an `ast::Expr` into an `anf::AExpr`, possibly adding bindings to
/// the context.
fn into_aexpr(expr: AstExpr, context: &mut Vec<(Symbol, Expr)>) -> AExpr {
    match convert_aexpr(expr) {
        Ok(aexpr) => aexpr,
        Err(expr) => {
            let sym = gensym();
            context.push((sym, expr.into()));
            AExpr::Var(sym)
        }
    }
}
