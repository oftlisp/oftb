use std::collections::{HashMap, HashSet};

use symbol::Symbol;

use anf::Module;
use error::Error;
use flatanf::AExpr;

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
            return Err(Error::DependencyLoopInModule(m.name));
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
    globals: HashMap<Symbol, Symbol>,
    locals: Vec<Symbol>,
}

impl Context {
    /// Brackets a function call with a push and a pop.
    pub fn bracket<F, T>(&mut self, name: Symbol, f: F) -> T
    where
        F: FnOnce(&mut Context) -> T,
    {
        self.push(name);
        let out = f(self);
        self.pop();
        out
    }

    /// Brackets a function call with several pushes and pops.
    pub fn bracket_many<F, I, T, U>(&mut self, names: I, f: F) -> U
    where
        F: FnOnce(&mut Context) -> U,
        I: IntoIterator<Item = T>,
        T: Into<Symbol>,
    {
        let mut n = 0;
        self.push_many(names.into_iter().map(|name| {
            n += 1;
            name
        }));
        let out = f(self);
        self.pop_many(n);
        out
    }

    /// Retrieves a value from the context, wrapping it into a
    /// `flatanf::AExpr`.
    pub fn get(&self, name: Symbol) -> Result<AExpr, Error> {
        if !self.locals.is_empty() {
            let off = self.locals.len() - 1;
            for n in 0..self.locals.len() {
                if self.locals[off - n] == name {
                    return Ok(AExpr::Local(n));
                }
            }
        }
        if let Some(&global) = self.globals.get(&name) {
            Ok(AExpr::Global(global))
        } else {
            Err(Error::NoSuchVar(name))
        }
    }

    /// Adds a value to the context.
    fn push(&mut self, name: Symbol) {
        self.locals.push(name);
    }

    /// Adds several values to the context.
    fn push_many<I, T>(&mut self, names: I)
    where
        I: IntoIterator<Item = T>,
        T: Into<Symbol>,
    {
        for s in names {
            self.push(s.into());
        }
    }

    /// Removes the most recently defined value from the context.
    fn pop(&mut self) {
        if self.locals.pop().is_none() {
            panic!("Popped from empty context");
        }
    }

    /// Removes the *n* most recently defined values from the context.
    fn pop_many(&mut self, n: usize) {
        for _ in 0..n {
            self.pop();
        }
    }
}

impl From<HashMap<Symbol, Symbol>> for Context {
    fn from(globals: HashMap<Symbol, Symbol>) -> Context {
        Context {
            globals,
            locals: Vec::new(),
        }
    }
}
