//! The module system.

mod metadata;
#[cfg(test)]
mod tests;

use std::collections::{HashMap, HashSet};
use std::path::Path;

use either::{Either, Left, Right};
use symbol::Symbol;

use anf::Module;
use error::Error;
use flatanf::Program;
pub use modules::metadata::{BinaryComponentMetadata, ComponentsMetadata,
                            DependencyMetadata, LibraryComponentMetadata,
                            PackageMetadata};

/// The module context.
///
/// The `Left` alternative represents a builtin module, while the `Right`
/// alternative is a loaded module.
#[derive(Clone, Debug)]
pub struct Modules(pub HashMap<Symbol, Either<HashSet<Symbol>, Module>>);

impl Modules {
    /// Creates a new `Modules` instance.
    pub fn new() -> Modules {
        Modules(HashMap::new())
    }

    /// Loads the modules visible in the given directory.
    ///
    /// This includes both the module declared in the directory and any
    /// dependencies.
    pub fn add_modules_from<P>(&mut self, path: P) -> Result<(), Error>
    where
        P: AsRef<Path>,
    {
        unimplemented!();
    }

    /// Loads a module from the given directory, without loading dependencies.
    fn add_module_from(&mut self, path: &Path) -> Result<(), Error> {
        unimplemented!()
    }

    /// Compiles a root module into a `flatanf::Program`.
    pub fn compile(self) -> Result<Program, Error> {
        let mut builtins = HashMap::new();
        let mut mods = Vec::new();
        for (name, module) in self.0 {
            match module {
                Left(m) => {
                    builtins.insert(name, m);
                }
                Right(m) => {
                    mods.push(m);
                }
            }
        }
        Program::from_modules(mods, builtins)
    }
}
