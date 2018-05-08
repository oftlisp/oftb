use std::collections::HashMap;

use symbol::Symbol;

use flatanf::{AExpr, CExpr, Expr};
use interpreter::{State, Value};
use interpreter::control::Control;
use interpreter::env::Env;
use interpreter::kont::Kont;
use interpreter::store::Store;

/// Evaluates by a single step.
pub fn step<'program>(
    state: State<'program>,
    globals: &HashMap<Symbol, Value>,
    store: &mut Store<'program>,
) -> State<'program> {
    match state {
        State::Running(c, mut e, mut k) => match c {
            Control::Normal(expr) => match *expr {
                Expr::AExpr(ref expr) => {
                    let val = atomic(expr, &e, globals, store);
                    match k.pop() {
                        Some(Kont::Let(expr)) => {
                            e.push(val.clone());
                            State::Running(Control::Normal(expr), e, k)
                        }
                        Some(Kont::Seq(expr)) => {
                            State::Running(Control::Normal(expr), e, k)
                        }
                        None => State::Halted(val),
                    }
                }
                Expr::CExpr(ref expr) => match *expr {
                    CExpr::Call(ref func, ref args) => {
                        let func = atomic(func, &e, globals, store);
                        let args = args.iter()
                            .map(|arg| atomic(arg, &e, globals, store))
                            .collect::<Vec<_>>();
                        unimplemented!()
                    }
                    CExpr::If(ref c, ref t, ref e) => unimplemented!(),
                    CExpr::LetRec(ref bound, ref body) => unimplemented!(),
                },
                Expr::Let(ref l, ref r) => {
                    k.push(Kont::Let(r));
                    State::Running(Control::Normal(l), e, k)
                }
                Expr::Seq(ref l, ref r) => {
                    k.push(Kont::Seq(r));
                    State::Running(Control::Normal(l), e, k)
                }
            },
        },
        State::Halted(v) => State::Halted(v),
    }
}

/// Evaluates an atomic expression to an value.
pub fn atomic<'program>(
    expr: &'program AExpr,
    env: &Env,
    globals: &HashMap<Symbol, Value>,
    store: &mut Store<'program>,
) -> Value {
    match *expr {
        AExpr::Global(name) => globals[&name],
        AExpr::Lambda(argn, ref body) => {
            Value::Closure(store.store_closure(argn, body, env.clone()))
        }
        AExpr::Literal(ref lit) => unimplemented!(),
        AExpr::Local(n) => env.local(n),
        AExpr::Vector(ref exprs) => {
            let vals = exprs
                .iter()
                .map(|expr| atomic(expr, env, globals, store))
                .collect::<Vec<_>>();
            unimplemented!()
        }
    }
}
