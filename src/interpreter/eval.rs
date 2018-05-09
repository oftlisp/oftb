//! Helper functions for evaluation.

use std::collections::HashMap;

use symbol::Symbol;

use Literal;
use flatanf::{AExpr, CExpr, Expr};
use interpreter::{State, Value};
use interpreter::control::Control;
use interpreter::env::Env;
use interpreter::kont::Kont;
use interpreter::store::Store;

/// Evaluates by a single step.
pub fn step<'program>(
    control: Control<'program>,
    mut env: Env,
    mut konts: Vec<Kont<'program>>,
    globals: &HashMap<Symbol, Value>,
    store: &mut Store<'program>,
) -> State<'program> {
    trace!("running {:?}", control);
    match control {
        Control::Normal(expr) => match *expr {
            Expr::AExpr(ref expr) => {
                let val = atomic(expr, &env, globals, store);
                kontinue(val, konts)
            }
            Expr::CExpr(ref expr) => match *expr {
                CExpr::Call(ref func, ref args) => {
                    let func = atomic(func, &env, globals, store);
                    let args = args.iter()
                        .map(|arg| atomic(arg, &env, globals, store))
                        .collect();
                    apply(func, args, store, konts)
                }
                CExpr::If(ref c, ref t, ref e) => {
                    let c = atomic(c, &env, globals, store);
                    let expr = if let Value::Nil = c { e } else { t };
                    State::Running(Control::Normal(expr), env, konts)
                }
                CExpr::LetRec(ref bound, ref body) => unimplemented!(),
            },
            Expr::Let(ref l, ref r) => {
                konts.push(Kont::Let(r, env.clone()));
                State::Running(Control::Normal(l), env, konts)
            }
            Expr::Seq(ref l, ref r) => {
                konts.push(Kont::Seq(r, env.clone()));
                State::Running(Control::Normal(l), env, konts)
            }
        },
    }
}

/// Calls a function.
pub fn apply<'program>(
    func: Value,
    args: Vec<Value>,
    store: &Store<'program>,
    konts: Vec<Kont<'program>>,
) -> State<'program> {
    match func {
        Value::Closure(clo_addr) => {
            let (argn, body, mut env) = store.get_closure(clo_addr);
            if argn != args.len() {
                unimplemented!("Bad argn, {} vs {}", argn, args.len());
            }
            env = env.push(func);
            for arg in args {
                env = env.push(arg);
            }
            State::Running(Control::Normal(body), env, konts)
        }
        _ => unimplemented!("Not callable: {:?}", func),
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
        AExpr::Literal(ref lit) => literal(lit, store),
        AExpr::Local(n) => env.local(n),
        AExpr::Vector(ref exprs) => {
            let vals = exprs
                .iter()
                .map(|expr| {
                    let val = atomic(expr, env, globals, store);
                    store.store(val)
                })
                .collect::<Vec<_>>();
            let (a, l) = store.store_vec(&vals);
            Value::Vector(a, l)
        }
    }
}

/// Builds a literal into the store.
pub fn literal<'program>(lit: &Literal, store: &mut Store<'program>) -> Value {
    match *lit {
        Literal::Byte(n) => Value::Byte(n),
        Literal::Bytes(ref bs) => {
            let (a, l) = store.store_bytes(bs);
            Value::Bytes(a, l)
        }
        Literal::Cons(ref h, ref t) => {
            let h = literal(h, store);
            let t = literal(t, store);
            Value::Cons(store.store(h), store.store(t))
        }
        Literal::Fixnum(n) => Value::Fixnum(n),
        Literal::Nil => Value::Nil,
        Literal::String(ref s) => {
            let (a, l) = store.store_str(s);
            Value::String(a, l)
        }
        Literal::Symbol(s) => Value::Symbol(s),
        Literal::Vector(ref vs) => {
            let vs = vs.iter()
                .map(|l| {
                    let v = literal(l, store);
                    store.store(v)
                })
                .collect::<Vec<_>>();
            let (a, l) = store.store_vec(&vs);
            Value::Vector(a, l)
        }
    }
}

/// Applies a value onto the top continuation of the continuation stack,
/// returning the new state.
pub fn kontinue<'program>(
    val: Value,
    mut konts: Vec<Kont<'program>>,
) -> State<'program> {
    match konts.pop() {
        Some(Kont::Let(expr, env)) => {
            let env = env.push(val.clone());
            State::Running(Control::Normal(expr), env, konts)
        }
        Some(Kont::Seq(expr, env)) => {
            State::Running(Control::Normal(expr), env, konts)
        }
        None => State::Halted(val),
    }
}
