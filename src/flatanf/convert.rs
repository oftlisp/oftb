use std::collections::{HashMap, HashSet};

use symbol::Symbol;

use anf::{AExpr as AnfAExpr, CExpr as AnfCExpr, Decl as AnfDecl,
          Expr as AnfExpr, Module};
use error::Error;
use flatanf::{AExpr, CExpr, Decl, Expr, Program};
use flatanf::util::{toposort_mods, Context};
use literal::Literal;

impl Program {
    /// Creates a flat `Program` from a bunch of `anf::Module`s.
    pub fn from_modules(
        mods: Vec<Module>,
        builtins: HashMap<Symbol, HashSet<Symbol>>,
    ) -> Result<Program, Error> {
        let mut code = Vec::new();
        let builtin_modules = builtins.keys().cloned().collect();
        let mut globals = builtins
            .into_iter()
            .flat_map(|(m, ds)| ds.into_iter().map(move |d| global(m, d)))
            .collect();

        toposort_mods(mods, builtin_modules, |m| {
            code.extend(compile_module(&mut globals, m)?);
            Ok(())
        })?;
        Ok(Program(code))
    }
}

fn compile_module(
    globals: &mut HashSet<Symbol>,
    m: Module,
) -> Result<Vec<Decl>, Error> {
    warn!("Compiling {:#?}", m);
    let Module {
        name: module_name,
        imports,
        exports,
        body,
    } = m;

    let mut context = {
        let mut ctx = imports
            .into_iter()
            .map(|(m, d)| (d, global(m, d)))
            .collect::<HashMap<_, _>>();
        for (_, g) in ctx.iter() {
            if !globals.contains(g) {
                return Err(Error::NonexistentImport(module_name, *g));
            }
        }
        ctx.extend(
            body.iter()
                .map(|decl| decl.name())
                .map(|name| (name, global(module_name, name))),
        );
        Context::from(ctx)
    };

    let decls = body.into_iter()
        .map(|d| compile_decl(&mut context, d))
        .collect::<Result<Vec<_>, _>>()?;

    let decl_names = decls.iter().map(|d| d.name()).collect::<HashSet<_>>();
    for e in exports {
        if !decl_names.contains(&e) {
            return Err(Error::MissingExport(module_name, e));
        }
        globals.insert(global(m.name, e));
    }
    Ok(decls)
}

fn compile_decl(context: &mut Context, decl: AnfDecl) -> Result<Decl, Error> {
    match decl {
        AnfDecl::Def(name, expr) => unimplemented!(),
        AnfDecl::Defn(name, args, body) => unimplemented!(),
    }
}

fn compile_expr(context: &mut Context, expr: AnfExpr) -> Result<Expr, Error> {
    unimplemented!("{:#?}", expr)
}

fn compile_cexpr(
    context: &mut Context,
    expr: AnfCExpr,
) -> Result<CExpr, Error> {
    unimplemented!("{:#?}", expr)
}

fn compile_aexpr(
    context: &mut Context,
    expr: AnfAExpr,
) -> Result<AExpr, Error> {
    unimplemented!("{:#?}", expr)
}

fn global(m: Symbol, d: Symbol) -> Symbol {
    format!("{}:{}", m, d).into()
}
