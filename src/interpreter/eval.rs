//! Helper functions for evaluation.

use std::collections::HashMap;

use symbol::Symbol;

use flatanf::{AExpr, CExpr, Expr};
use interpreter::{Control, Env, Intrinsic, Kont, State, Store, Value};

/// Evaluates by a single step.
pub fn step<'program>(
    control: Control<'program>,
    mut env: Env,
    globals: &HashMap<Symbol, Value>,
    store: &mut Store<'program>,
    mut konts: Vec<Kont<'program>>,
) -> State<'program> {
    match control {
        Control::Normal(expr) => match *expr {
            Expr::AExpr(ref expr) => {
                let val = atomic(expr, &env, globals, store);
                kontinue(val, store, konts)
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
                CExpr::LetRec(ref lambdas, ref body) => {
                    let mut addrs = Vec::new();
                    for &(name, argn, ref body) in lambdas {
                        let a = store.store_closure(argn, body, Some(name), Env::new());
                        addrs.push(a);
                        env = env.push(Value::Closure(a));
                    }
                    for a in addrs {
                        store.mutate_closure_env(a, env.clone());
                    }
                    State::Running(Control::Normal(body), env, konts)
                }
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
    store: &mut Store<'program>,
    konts: Vec<Kont<'program>>,
) -> State<'program> {
    match func {
        Value::Closure(clo_addr) => {
            let (argn, body, name, mut env) = store.get_closure(clo_addr);
            if argn != args.len() {
                if let Some(name) = name {
                    unimplemented!("Bad argn in call to `{}', {} vs {}", name, argn, args.len());
                } else {
                    unimplemented!("Bad argn in call to a closure, {} vs {}", argn, args.len());
                }
            }
            for arg in args {
                env = env.push(arg);
            }
            State::Running(Control::Normal(body), env, konts)
        }
        Value::Intrinsic(Intrinsic(f)) => f(args, store, konts),
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
        AExpr::GetMethod(ref type_, name) => match atomic(type_, env, globals, store) {
            Value::Symbol(type_) => globals
                .get(&format!("{}#{}", type_, name).into())
                .cloned()
                .unwrap_or_else(|| panic!("No such method {} for type {}", name, type_)),
            value => panic!("Invalid type to get-method {}", value.display(store, false)),
        },
        AExpr::Global(name) => globals
            .get(&name)
            .cloned()
            .unwrap_or_else(|| panic!("Unknown global: {}", name)),
        AExpr::Lambda(name, argn, ref body) => {
            Value::Closure(store.store_closure(argn, body, name, env.clone()))
        }
        AExpr::Literal(ref lit) => store.store_literal(lit),
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

/// Applies a value onto the top continuation of the continuation stack,
/// returning the new state.
pub fn kontinue<'program>(
    val: Value,
    store: &mut Store<'program>,
    mut konts: Vec<Kont<'program>>,
) -> State<'program> {
    match konts.pop() {
        Some(Kont::Let(expr, env)) => {
            let env = env.push(val.clone());
            State::Running(Control::Normal(expr), env, konts)
        }
        Some(Kont::MakeVector(cur, last, func, mut acc)) => {
            debug_assert_eq!(cur, acc.len());
            acc.push(store.store(val));
            if cur == last {
                let (addr, len) = store.store_vec(&acc);
                debug_assert_eq!(len, cur + 1);
                kontinue(Value::Vector(addr, len), store, konts)
            } else {
                let cur = cur + 1;
                konts.push(Kont::MakeVector(cur, last, func, acc));
                apply(func, vec![Value::Fixnum(cur as isize)], store, konts)
            }
        }
        Some(Kont::Seq(expr, env)) => State::Running(Control::Normal(expr), env, konts),
        None => State::Halted(val),
    }
}
