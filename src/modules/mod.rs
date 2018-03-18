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
                            DependencyMetadata, Error as MetadataError,
                            LibraryComponentMetadata, PackageMetadata};

/// The module context.
///
/// The `Left` alternative represents a builtin module, while the `Right`
/// alternative is a loaded module.
#[derive(Clone, Debug)]
pub struct Modules(
    pub HashMap<Symbol, (PackageMetadata, Either<HashSet<Symbol>, Module>)>,
);

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
        let path = path.as_ref();

        let root_meta = self.load_metadata_from(path)?;
        for dep in root_meta.dependencies {
            unimplemented!("{:#?}", dep);
        }

        self.add_module_from(path)
    }

    /// Loads a module from the given directory, without loading dependencies.
    /// However, this will panic if the dependencies are not already met.
    fn add_module_from(&mut self, path: &Path) -> Result<(), Error> {
        let meta = self.load_metadata_from(path)?;
        unimplemented!("{:#?}", meta)
    }

    /// Compiles a root module into a `flatanf::Program`.
    pub fn compile(self) -> Result<Program, Error> {
        let mut builtins = HashMap::new();
        let mut mods = Vec::new();
        for (name, (_meta, module)) in self.0 {
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

    /// Loads metadata from the module in the given directory.
    fn load_metadata_from(
        &self,
        path: &Path,
    ) -> Result<PackageMetadata, Error> {
        use parser::parse_file;

        let lits = parse_file(path.join("package.oftd"))?;
        PackageMetadata::from_literals(lits).map_err(Into::into)
    }
}
