use std::collections::{HashMap, HashSet};

use failure::ResultExt;
use symbol::Symbol;

use anf::{AExpr as AnfAExpr, CExpr as AnfCExpr, Decl as AnfDecl,
          Expr as AnfExpr, Module};
use error::{Error, ErrorKind};
use flatanf::util::{toposort_mods, Context};
use flatanf::{AExpr, CExpr, Expr, Program};

impl Program {
    /// Creates a `Program` from a bunch of `anf::Module`s.
    pub fn from_modules(
        mods: Vec<Module>,
        builtins: HashMap<Symbol, HashSet<Symbol>>,
    ) -> Result<Program, Error> {
        let mut decls = Vec::new();
        let builtin_modules = builtins.keys().cloned().collect();
        let mut intrinsics = builtins
            .into_iter()
            .flat_map(|(m, ds)| ds.into_iter().map(move |d| global(m, d)))
            .collect::<HashSet<Symbol>>();
        let mut globals = intrinsics.clone();

        toposort_mods(mods, builtin_modules, |m| {
            let name = m.name;
            let m = compile_module(&mut globals, m)
                .context(ErrorKind::CouldntCompileModule(name))?;
            decls.extend(m);
            Ok(())
        })?;

        let free = freevars(&decls);
        intrinsics.retain(|x| free.contains(x));
        Ok(Program {
            decls,
            intrinsics,
        })
    }
}

fn compile_module(
    globals: &mut HashSet<Symbol>,
    m: Module,
) -> Result<Vec<(Symbol, Expr)>, Error> {
    let Module {
        name: module_name,
        imports,
        exports,
        body,
        attrs: _,
    } = m;

    let mut context = {
        let mut ctx = imports
            .into_iter()
            .map(|(m, d)| (d, global(m, d)))
            .collect::<HashMap<_, _>>();
        for (_, g) in ctx.iter() {
            if !globals.contains(g) {
                return Err(ErrorKind::NonexistentImport(module_name, *g).into());
            }
        }
        Context::from(ctx)
    };

    // So the logic here is a bit complex...
    //
    // We want to populate the context with all the defns, until we hit a def.
    // Then we actually compile all the defns, then the def. Rinse & repeat.
    let mut batched_defns = Vec::new();
    let mut decls = Vec::new();
    let compile_batched_defns = |batched_defns: &mut Vec<_>,
                                 decls: &mut Vec<_>,
                                 context: &mut Context|
     -> Result<(), Error> {
        batched_defns.iter().for_each(|&(name, _, _)| {
            context.add_global(name, global(module_name, name))
        });
        let defns = batched_defns
            .drain(..)
            .map(|(n, a, b)| {
                compile_decl(module_name, context, AnfDecl::Defn(n, a, b))
            })
            .collect::<Result<Vec<_>, _>>()?;
        decls.extend(defns);
        Ok(())
    };

    for decl in body {
        match decl {
            AnfDecl::Defn(name, args, body) => {
                batched_defns.push((name, args, body));
            }
            AnfDecl::Def(name, expr) => {
                compile_batched_defns(
                    &mut batched_defns,
                    &mut decls,
                    &mut context,
                )?;
                let (global_name, expr) = compile_decl(
                    module_name,
                    &mut context,
                    AnfDecl::Def(name, expr),
                )?;
                decls.push((global_name, expr));
                context.add_global(name, global_name);
            }
        }
    }
    compile_batched_defns(&mut batched_defns, &mut decls, &mut context)?;

    let decl_names = decls
        .iter()
        .map(|&(name, _)| name)
        .collect::<HashSet<_>>();
    for e in exports {
        let e = global(module_name, e);
        if !decl_names.contains(&e) {
            return Err(ErrorKind::MissingExport(module_name, e).into());
        }
        globals.insert(e);
    }
    Ok(decls)
}

fn compile_decl(
    mod_name: Symbol,
    context: &mut Context,
    decl: AnfDecl,
) -> Result<(Symbol, Expr), Error> {
    match decl {
        AnfDecl::Def(name, expr) => {
            let name = global(mod_name, name);
            let expr = compile_expr(context, expr)?;
            Ok((name, expr))
        }
        AnfDecl::Defn(name, args, body) => {
            let body = context.bracket_many(args.iter().cloned(), |context| {
                compile_expr(context, body)
            })?;
            let name = global(mod_name, name);
            let expr = Expr::AExpr(AExpr::Lambda(
                Some(name),
                args.len(),
                Box::new(body),
            ));
            Ok((name, expr))
        }
    }
}

fn compile_expr(context: &mut Context, expr: AnfExpr) -> Result<Expr, Error> {
    match expr {
        AnfExpr::AExpr(expr) => compile_aexpr(context, expr).map(Expr::AExpr),
        AnfExpr::CExpr(expr) => compile_cexpr(context, expr).map(Expr::CExpr),
        AnfExpr::Let(name, bound, body) => {
            let bound = compile_expr(context, *bound)?;
            let body =
                context.bracket(name, |context| compile_expr(context, *body))?;
            Ok(Expr::Let(Box::new(bound), Box::new(body)))
        }
        AnfExpr::Seq(e1, e2) => {
            let e1 = compile_expr(context, *e1)?;
            let e2 = compile_expr(context, *e2)?;
            Ok(Expr::Seq(Box::new(e1), Box::new(e2)))
        }
    }
}

fn compile_cexpr(
    context: &mut Context,
    expr: AnfCExpr,
) -> Result<CExpr, Error> {
    match expr {
        AnfCExpr::Call(func, args) => {
            let func = compile_aexpr(context, func)?;
            let args = args.into_iter()
                .map(|arg| compile_aexpr(context, arg))
                .collect::<Result<_, _>>()?;
            Ok(CExpr::Call(func, args))
        }
        AnfCExpr::If(c, t, e) => {
            let c = compile_aexpr(context, c)?;
            let t = compile_expr(context, *t)?;
            let e = compile_expr(context, *e)?;
            Ok(CExpr::If(c, Box::new(t), Box::new(e)))
        }
        AnfCExpr::LetRec(lambdas, body) => {
            let names = lambdas
                .iter()
                .map(|&(n, _, _)| n)
                .collect::<Vec<_>>();
            context.bracket_many(names, |context| {
                let lambdas = lambdas
                    .into_iter()
                    .map(|(name, args, body)| {
                        let argn = args.len();
                        let body = context.bracket_many(args, |context| {
                            compile_expr(context, body)
                        })?;
                        Ok((name, argn, body))
                    })
                    .collect::<Result<Vec<_>, Error>>()?;
                let body = compile_expr(context, *body)?;
                Ok(CExpr::LetRec(lambdas, Box::new(body)))
            })
        }
    }
}

fn compile_aexpr(
    context: &mut Context,
    expr: AnfAExpr,
) -> Result<AExpr, Error> {
    match expr {
        AnfAExpr::Lambda(name, args, body) => {
            let argn = args.len();
            let body = context
                .bracket_many(args, |context| compile_expr(context, *body))?;
            Ok(AExpr::Lambda(name, argn, Box::new(body)))
        }
        AnfAExpr::Literal(lit) => Ok(AExpr::Literal(lit)),
        AnfAExpr::Var(var) if var.contains(':') => {
            // This is checked by the globals_exist sanity check.
            Ok(AExpr::Global(var))
        }
        AnfAExpr::Var(var) => context.get(var),
        AnfAExpr::Vector(exprs) => {
            let exprs = exprs
                .into_iter()
                .map(|e| compile_aexpr(context, e))
                .collect::<Result<_, _>>()?;
            Ok(AExpr::Vector(exprs))
        }
    }
}

/// Returns the free variables of a set of decls.
pub fn freevars(decls: &[(Symbol, Expr)]) -> HashSet<Symbol> {
    let found = decls
        .iter()
        .flat_map(|&(_, ref expr)| expr.global_vars())
        .collect::<HashSet<_>>();
    let declared = decls.iter().map(|&(name, _)| name).collect();
    found.difference(&declared).cloned().collect()
}

fn global(m: Symbol, d: Symbol) -> Symbol {
    format!("{}:{}", m, d).into()
}
