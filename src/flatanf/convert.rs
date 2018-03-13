use std::collections::{BTreeMap, BTreeSet};

use symbol::Symbol;

use anf::{AExpr as AnfAExpr, CExpr as AnfCExpr, Decl as AnfDecl,
          Expr as AnfExpr, Module as AnfModule};
use error::Error;
use flatanf::{AExpr, CExpr, Decl, Expr, Program};
use literal::Literal;

impl Program {
    /// Creates a flat `Program` from a bunch of `anf::Module`s.
    pub fn from_modules(
        mods: Vec<AnfModule>,
        builtins: BTreeMap<Symbol, BTreeSet<Symbol>>,
    ) -> Result<Program, Error> {
        println!();
        toposort_mods(mods, builtins.keys().cloned().collect(), |m| {
            info!("{:?}", m);
        })?;
        let mut known = builtins;
        unimplemented!("known = {:?}", known)
    }
}

/// A topological sort implemented as a traversal. `f` is called once for each
/// module in `mods`. If there is a circular dependency, an error will be
/// returned.
fn toposort_mods<F: FnMut(AnfModule)>(
    mut mods: Vec<AnfModule>,
    builtins: BTreeSet<Symbol>,
    mut f: F,
) -> Result<(), Error> {
    fn traverse<F: FnMut(AnfModule)>(
        m: AnfModule,
        mods: &mut Vec<AnfModule>,
        open: &mut BTreeSet<Symbol>,
        closed: &mut BTreeSet<Symbol>,
        f: &mut F,
    ) -> Result<(), Error> {
        if closed.contains(&m.name) {
            return Ok(());
        } else if !open.insert(m.name) {
            return Err(Error::DependencyLoop(m.name));
        }
        for &(name, _) in &m.imports {
            let i = mods.binary_search_by_key(&name, |m| m.name)
                .map_err(|_| Error::NonexistentModule(name))?;
            let m = mods.remove(i);
            traverse(m, mods, open, closed, f)?;
        }
        closed.insert(m.name);
        f(m);
        Ok(())
    }

    let mut open = BTreeSet::new();
    let mut closed = builtins;

    mods.sort_by_key(|m| m.name);
    loop {
        let next = if let Some(next) = mods.pop() {
            next
        } else {
            return Ok(());
        };

        traverse(next, &mut mods, &mut open, &mut closed, &mut f)?;
    }
}
