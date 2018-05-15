use flatanf::{AExpr, CExpr, Expr, Program};

/// Checks that all referenced local variables would actually exist.
///
/// This check panics if it fails -- a failed test here indicates a compiler
/// error.
pub fn locals_valid(program: &Program) {
    for &(_, ref e) in &program.decls {
        expr(0, e);
    }
}

fn expr(depth: usize, e: &Expr) {
    match *e {
        Expr::AExpr(ref e) => aexpr(depth, e),
        Expr::CExpr(ref e) => cexpr(depth, e),
        Expr::Let(ref x, ref y) => {
            expr(depth, x);
            expr(depth + 1, y);
        }
        Expr::Seq(ref x, ref y) => {
            expr(depth, x);
            expr(depth, y);
        }
    }
}

fn aexpr(depth: usize, e: &AExpr) {
    match *e {
        AExpr::Lambda(argn, ref body) => expr(depth + argn, body),
        AExpr::Local(n) => assert!(n < depth),
        AExpr::Vector(ref es) => {
            for e in es {
                aexpr(depth, e);
            }
        }
        _ => {}
    }
}

fn cexpr(depth: usize, e: &CExpr) {
    match *e {
        CExpr::Call(ref func, ref args) => {
            aexpr(depth, func);
            for a in args {
                aexpr(depth, a);
            }
        }
        CExpr::If(ref c, ref t, ref e) => {
            aexpr(depth, c);
            expr(depth, t);
            expr(depth, e);
        }
        CExpr::LetRec(ref bound, ref body) => {
            let depth = depth + bound.len();
            for &(argn, ref body) in bound {
                expr(depth + argn, body);
            }
            expr(depth, body);
        }
    }
}
