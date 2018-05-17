use std::collections::{HashMap, HashSet};

use symbol::Symbol;

use anf::Module;
use error::{Error, ErrorKind};
use flatanf::AExpr;

/// A topological sort implemented as a traversal. `f` is called once for each
/// module in `mods`. If there is a circular dependency, an error will be
/// returned.
pub fn toposort_mods<F: FnMut(Module) -> Result<(), Error>>(
    mut mods: Vec<Module>,
    builtins: HashSet<Symbol>,
    mut f: F,
) -> Result<(), Error> {
    // TODO: It ought to be possible to use a binary search here. For some
    // reason, it doesn't work...
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
            return Err(ErrorKind::DependencyLoopInModule(m.name).into());
        }
        for &(name, _) in &m.imports {
            if !closed.contains(&name) {
                let i = mods.iter()
                    .position(|m| name == m.name)
                    .ok_or_else(|| ErrorKind::NonexistentModule(name))?;
                let m = mods.remove(i);
                traverse(m, mods, open, closed, f)?;
            }
        }
        closed.insert(m.name);
        f(m)
    }

    let mut open = HashSet::with_capacity(mods.len());
    let mut closed = builtins;

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
#[derive(Clone, Debug, Default)]
pub struct Context {
    globals: HashMap<Symbol, Symbol>,
    locals: Vec<Symbol>,
}

impl Context {
    /// Adds a global binding.
    ///
    ///  - `name` is the "local name," for example `map`.
    ///  - `fully_qualified` is the "global name," for example `std/prelude:map`.
    pub fn add_global(&mut self, name: Symbol, fully_qualified: Symbol) {
        self.globals.insert(name, fully_qualified);
    }

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
            Err(ErrorKind::NoSuchVar(name).into())
        }
    }

    /// Adds a binding to the context.
    fn push(&mut self, name: Symbol) {
        self.locals.push(name);
    }

    /// Adds several bindings to the context.
    fn push_many<I, T>(&mut self, names: I)
    where
        I: IntoIterator<Item = T>,
        T: Into<Symbol>,
    {
        for s in names {
            self.push(s.into());
        }
    }

    /// Removes the most recently defined binding from the context.
    fn pop(&mut self) {
        if self.locals.pop().is_none() {
            panic!("Popped from empty context");
        }
    }

    /// Removes the *n* most recently defined bindings from the context.
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
