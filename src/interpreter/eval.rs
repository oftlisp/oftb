use flatanf::{AExpr, CExpr, Expr};
use interpreter::{State, Value};
use interpreter::control::Control;
use interpreter::env::Env;
use interpreter::kont::Kont;
use interpreter::store::Store;

/// Evaluates by a single step.
pub fn step<'program>(store: State<'program>) -> State<'program> {
    match store {
        State::Running(c, mut e, mut s, mut k) => match c {
            Control::Normal(expr) => match *expr {
                Expr::AExpr(ref expr) => {
                    let val = atomic(expr, &e, &mut s);
                    match k.pop() {
                        Some(Kont::Let(expr)) => {
                            e.push(val.clone());
                            State::Running(Control::Normal(expr), e, s, k)
                        }
                        Some(Kont::Seq(expr)) => {
                            State::Running(Control::Normal(expr), e, s, k)
                        }
                        None => State::Halted(val),
                    }
                }
                Expr::CExpr(ref e) => match *e {
                    CExpr::Call(ref func, ref args) => unimplemented!(),
                    CExpr::If(ref c, ref t, ref e) => unimplemented!(),
                    CExpr::LetRec(ref bound, ref body) => unimplemented!(),
                },
                Expr::Let(ref l, ref r) => {
                    k.push(Kont::Let(r));
                    State::Running(Control::Normal(l), e, s, k)
                }
                Expr::Seq(ref l, ref r) => {
                    k.push(Kont::Seq(r));
                    State::Running(Control::Normal(l), e, s, k)
                }
            },
        },
        State::Halted(v) => State::Halted(v),
    }
}

/// Evaluates an atomic expression to an value.
pub fn atomic<'program>(
    expr: &'program AExpr,
    env: &Env<'program>,
    store: &mut Store<'program>,
) -> Value<'program> {
    match *expr {
        AExpr::Global(name) => env.global(name),
        AExpr::Lambda(argn, ref body) => unimplemented!(),
        AExpr::Literal(ref lit) => unimplemented!(),
        AExpr::Local(n) => env.local(n),
        AExpr::Vector(ref exprs) => unimplemented!(),
    }
}
