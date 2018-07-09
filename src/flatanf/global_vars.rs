use std::collections::BTreeSet;

use symbol::Symbol;

use flatanf::{AExpr, CExpr, Expr};

impl Expr {
    /// Returns the global variables used by the given expression or its
    /// subexpressions.
    pub fn global_vars(&self) -> BTreeSet<Symbol> {
        match *self {
            Expr::AExpr(ref e) => e.global_vars(),
            Expr::CExpr(ref e) => e.global_vars(),
            Expr::Let(ref a, ref b) => a.global_vars().into_iter().chain(b.global_vars()).collect(),
            Expr::Seq(ref a, ref b) => a.global_vars().into_iter().chain(b.global_vars()).collect(),
        }
    }
}

impl CExpr {
    /// Returns the global variables used by the given expression or its
    /// subexpressions.
    pub fn global_vars(&self) -> BTreeSet<Symbol> {
        match *self {
            CExpr::Call(ref func, ref args) => args.iter()
                .flat_map(|a| a.global_vars())
                .chain(func.global_vars())
                .collect(),
            CExpr::If(ref c, ref t, ref e) => c.global_vars()
                .into_iter()
                .chain(t.global_vars())
                .chain(e.global_vars())
                .collect(),
            CExpr::LetRec(ref bound, ref body) => bound
                .iter()
                .flat_map(|&(_, _, ref b)| b.global_vars())
                .chain(body.global_vars())
                .collect(),
        }
    }
}

impl AExpr {
    /// Returns the global variables used by the given expression or its
    /// subexpressions.
    pub fn global_vars(&self) -> BTreeSet<Symbol> {
        match *self {
            AExpr::GetMethod(ref type_, _) => type_.global_vars(),
            AExpr::Global(name) => {
                let mut s = BTreeSet::new();
                s.insert(name);
                s
            }
            AExpr::Lambda(_, _, ref body) => body.global_vars(),
            AExpr::Literal(_) => BTreeSet::new(),
            AExpr::Local(_) => BTreeSet::new(),
            AExpr::Vector(ref vec) => vec.iter().flat_map(|e| e.global_vars()).collect(),
        }
    }
}
