use std::fmt::{Display, Formatter, Result as FmtResult};

use flatanf::{AExpr, CExpr, Expr, Program};

impl Display for Program {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        write!(fmt, "intrinsics {{")?;
        let mut first = true;
        for &i in &self.intrinsics {
            if first {
                first = false;
            } else {
                write!(fmt, ", ")?;
            }
            write!(fmt, "{}", i)?;
        }
        write!(fmt, "}};\n\n\n")?;

        for &(name, ref expr) in &self.decls {
            writeln!(fmt, "{} = {}\n", name, expr)?;
        }
        Ok(())
    }
}

impl Display for Expr {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        match *self {
            Expr::AExpr(ref expr) => write!(fmt, "{}", expr),
            Expr::CExpr(ref expr) => write!(fmt, "{}", expr),
            Expr::Let(ref a, ref b) => write!(fmt, "let {} in\n{}", a, b),
            Expr::Seq(ref a, ref b) => write!(fmt, "{};\n{}", a, b),
        }
    }
}

impl Display for CExpr {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        match *self {
            CExpr::Call(ref func, ref args) => {
                write!(fmt, "({}", func)?;
                for arg in args {
                    write!(fmt, " {}", arg)?;
                }
                write!(fmt, ")")
            }
            CExpr::If(ref c, ref t, ref e) => {
                write!(fmt, "if {} then {} else {} endif", c, t, e)
            }
            CExpr::LetRec(ref bound, ref body) => {
                writeln!(fmt, "letrec")?;
                for (argn, ref body) in bound {
                    writeln!(fmt, "  lam({}). {}", argn, body)?;
                }
                write!(fmt, "{}", body)
            }
        }
    }
}

impl Display for AExpr {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        match *self {
            AExpr::Global(name) => write!(fmt, "{}", name),
            AExpr::Lambda(argn, ref body) => {
                write!(fmt, "lam({}). {}", argn, body)
            }
            AExpr::Literal(ref lit) => write!(fmt, "{}", lit),
            AExpr::Local(n) => write!(fmt, "${}", n),
            AExpr::Vector(ref vals) => {
                write!(fmt, "[")?;

                let mut first = true;
                for val in vals {
                    if first {
                        first = false;
                    } else {
                        write!(fmt, ", ")?;
                    }
                    write!(fmt, "{}", val)?;
                }

                write!(fmt, "]")
            }
        }
    }
}
