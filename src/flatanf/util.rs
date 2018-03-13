use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use symbol::Symbol;

use error::Error;
use anf::Module;

/// A topological sort implemented as a traversal. `f` is called once for each
/// module in `mods`. If there is a circular dependency, an error will be
/// returned.
pub fn toposort_mods<F: FnMut(Module) -> Result<(), Error>>(
    mut mods: Vec<Module>,
    builtins: HashSet<Symbol>,
    mut f: F,
) -> Result<(), Error> {
    fn traverse<F: FnMut(Module) -> Result<(), Error>>(
        m: Module,
        mods: &mut Vec<Module>,
        open: &mut HashSet<Symbol>,
        closed: &mut HashSet<Symbol>,
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
        f(m)
    }

    let mut open = HashSet::with_capacity(mods.len());
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

/// A context for converting to use De Bruijn indices.
#[derive(Clone, Debug)]
pub struct Context {
    inner: Rc<ContextInner>,
}

#[derive(Debug)]
enum ContextInner {
    End(HashMap<Symbol, Symbol>),
    Local(Symbol, Rc<ContextInner>),
}

impl Context {
    fn get(&self, name: Symbol) -> Result<usize, Error> {
        let mut acc = 0;
        let mut ctx = self.inner.as_ref();
        loop {
            match *ctx {
                ContextInner::End(_) => return Err(Error::NoSuchVar(name)),
                ContextInner::Local(n, _) if n == name => return Ok(acc),
                ContextInner::Local(_, ref c) => {
                    acc += 1;
                    ctx = &**c;
                }
            }
        }
    }

    fn push(&mut self, name: Symbol) {
        self.inner = Rc::new(ContextInner::Local(name, self.inner.clone()));
    }

    fn pop(&mut self) {
        let next = match *self.inner {
            ContextInner::End(_) => panic!("Popped from empty context"),
            ContextInner::Local(_, ref c) => c.clone(),
        };
        self.inner = next;
    }
}

impl From<HashMap<Symbol, Symbol>> for Context {
    fn from(m: HashMap<Symbol, Symbol>) -> Context {
        let inner = Rc::new(ContextInner::End(m));
        Context { inner }
    }
}
