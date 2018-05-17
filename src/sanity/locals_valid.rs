use std::fmt::{Display, Formatter, Result as FmtResult};

use flatanf::{AExpr, CExpr, Expr, Program};

/// Checks that all referenced local variables would actually exist.
///
/// This check panics if it fails -- a failed test here indicates a compiler
/// error.
pub fn locals_valid(program: &Program) {
    for &(name, ref e) in &program.decls {
        let fr = check_expr(0, e);
        if fr.is_err() {
            error!("In decl {}:\n\n{}", name, fr);
            panic!("In decl {}:\n\n{}", name, fr)
        }
    }
}

fn check_expr<'a>(depth: usize, expr: &'a Expr) -> FailRecord<'a> {
    match *expr {
        Expr::AExpr(ref e) => check_aexpr(depth, e),
        Expr::CExpr(ref e) => check_cexpr(depth, e),
        Expr::Let(ref x, ref y) => {
            check_expr(depth, x).or_else(|| check_expr(depth + 1, y))
        }
        Expr::Seq(ref x, ref y) => {
            check_expr(depth, x).or_else(|| check_expr(depth, y))
        }
    }.chain(expr)
}

fn check_aexpr<'a>(depth: usize, expr: &'a AExpr) -> FailRecord<'a> {
    match *expr {
        AExpr::Lambda(argn, ref body) => check_expr(depth + argn, body),
        AExpr::Local(n) if n >= depth => FailRecord::err(n, depth),
        AExpr::Local(_) => FailRecord::ok(),
        AExpr::Vector(ref es) => {
            FailRecord::all(es.iter().map(|e| check_aexpr(depth, e)))
        }
        _ => FailRecord::ok(),
    }.chain_a(expr)
}

fn check_cexpr<'a>(depth: usize, expr: &'a CExpr) -> FailRecord<'a> {
    match *expr {
        CExpr::Call(ref func, ref args) => check_aexpr(depth, func).or_else(
            || FailRecord::all(args.iter().map(|a| check_aexpr(depth, a))),
        ),
        CExpr::If(ref c, ref t, ref e) => check_aexpr(depth, c)
            .or_else(|| check_expr(depth, t).or_else(|| check_expr(depth, e))),
        CExpr::LetRec(ref bound, ref body) => {
            let depth = depth + bound.len();
            FailRecord::all(
                bound
                    .iter()
                    .map(|&(argn, ref body)| check_expr(depth + argn, body)),
            ).or_else(|| check_expr(depth, body))
        }
    }.chain_c(expr)
}

enum FailRecord<'a> {
    CauseA(&'a AExpr, Box<FailRecord<'a>>),
    CauseC(&'a CExpr, Box<FailRecord<'a>>),
    CauseE(&'a Expr, Box<FailRecord<'a>>),
    NoError,
    RootCause(usize, usize),
}

impl<'a> FailRecord<'a> {
    fn all(iter: impl IntoIterator<Item = Self>) -> Self {
        let mut iter = iter.into_iter();
        while let Some(fr) = iter.next() {
            if let &FailRecord::NoError = fr.root() {
                continue;
            }
            return fr;
        }
        FailRecord::NoError
    }
    fn chain(self, expr: &'a Expr) -> Self {
        if self.is_err() {
            FailRecord::CauseE(expr, Box::new(self))
        } else {
            self
        }
    }
    fn chain_a(self, expr: &'a AExpr) -> Self {
        if self.is_err() {
            FailRecord::CauseA(expr, Box::new(self))
        } else {
            self
        }
    }
    fn chain_c(self, expr: &'a CExpr) -> Self {
        if self.is_err() {
            FailRecord::CauseC(expr, Box::new(self))
        } else {
            self
        }
    }
    fn err(var: usize, depth: usize) -> Self {
        FailRecord::RootCause(var, depth)
    }
    fn is_err(&self) -> bool {
        if let &FailRecord::NoError = self.root() {
            false
        } else {
            true
        }
    }
    fn ok() -> Self {
        FailRecord::NoError
    }
    fn or_else(self, f: impl FnOnce() -> Self) -> Self {
        match self {
            FailRecord::NoError => f(),
            err => err,
        }
    }
    fn root(&self) -> &Self {
        let mut fr = self;
        loop {
            let next = match *fr {
                FailRecord::CauseA(_, ref tl) => tl,
                FailRecord::CauseC(_, ref tl) => tl,
                FailRecord::CauseE(_, ref tl) => tl,
                ref fr => return fr,
            };
            fr = next;
        }
    }
}

impl<'a> Display for FailRecord<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        match *self {
            FailRecord::CauseA(expr, ref tl) => {
                write!(fmt, "In expr {}:\n\n{}", expr, tl)
            }
            FailRecord::CauseC(expr, ref tl) => {
                write!(fmt, "In expr {}:\n\n{}", expr, tl)
            }
            FailRecord::CauseE(expr, ref tl) => {
                write!(fmt, "In expr {}:\n\n{}", expr, tl)
            }
            FailRecord::NoError => write!(fmt, "No error"),
            FailRecord::RootCause(var, depth) => write!(
                fmt,
                "Tried to access local {} at depth {}",
                var, depth
            ),
        }
    }
}
