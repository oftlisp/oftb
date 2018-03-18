//! The module system.

mod metadata;
#[cfg(test)]
mod tests;

use std::collections::{HashMap, HashSet};
use std::fmt::Write;
use std::path::{Path, PathBuf};

use symbol::Symbol;

use anf::Module;
use error::Error;
use flatanf::Program;
pub use modules::metadata::{BinaryComponentMetadata, ComponentsMetadata,
                            DependencyMetadata, Error as MetadataError,
                            LibraryComponentMetadata, PackageMetadata};
use parser::parse_file;

#[derive(Clone, Debug)]
enum Package {
    Builtins(HashSet<Symbol>),
    Filesystem(PackageMetadata, Vec<Module>),
}

/// The module context.
///
/// The `Left` alternative represents a builtin module, while the `Right`
/// alternative is a loaded module.
#[derive(Clone, Debug)]
pub struct Packages(HashMap<Symbol, Package>);

impl Packages {
    /// Creates a new `Packages` instance.
    pub fn new() -> Packages {
        Packages(HashMap::new())
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
        self.add_package_from(root_meta.name, path)
    }

    /// Loads a package from the given directory, without loading dependencies.
    /// However, this will panic if the dependencies are not already met.
    ///
    /// This is useful for loading, for example, the standard library.
    pub fn add_package_from(
        &mut self,
        package_name: Symbol,
        path: &Path,
    ) -> Result<(), Error> {
        let meta = self.load_metadata_from(path)?;
        if package_name != meta.name {
            return Err(Error::MisnamedPackage(package_name, meta.name));
        } else if meta.components.library.is_none() {
            return Err(Error::DependencyMustExportLib(package_name));
        }

        let mut modules = Vec::new();
        let src_path = path.join("src");
        println!("{}", src_path.display());
        let lib_oft_path = src_path.join("lib.oft");

        fn crawl(
            modules: &mut Vec<Module>,
            mod_stack: &mut Vec<Symbol>,
            base: PathBuf,
            lib_oft_path: &Path,
        ) -> Result<(), Error> {
            for entry in base.read_dir()? {
                let entry = entry?;
                let file_type = entry.file_type()?;
                if file_type.is_dir() {
                    let name = entry
                        .file_name()
                        .into_string()
                        .expect("Non-Unicode source directory name...")
                        .into();
                    mod_stack.push(name);
                    crawl(modules, mod_stack, entry.path(), lib_oft_path)?;
                    assert_eq!(mod_stack.pop(), Some(name));
                } else if file_type.is_file() {
                    let file_name: PathBuf = entry.file_name().into();
                    if file_name.extension() != Some("oft".as_ref()) {
                        continue;
                    }
                    let file_name = file_name.file_stem().unwrap();

                    let mut name = String::new();
                    for &mod_part in mod_stack.iter() {
                        write!(name, "{}/", mod_part).unwrap();
                    }
                    let path = entry.path();
                    if path == lib_oft_path {
                        assert_eq!(name.pop(), Some('/'));
                    } else {
                        name += file_name
                            .to_str()
                            .expect("Non-Unicode source file name...");
                    }
                    let name = name.into();

                    let module = Packages::load_module(&path)?;
                    if name != module.name {
                        return Err(Error::MisnamedModule(name, module.name));
                    }

                    modules.push(module);
                } else {
                    warn!(
                        "Source file `{}' is neither directory nor file",
                        entry.path().display()
                    );
                }
            }
            Ok(())
        }
        crawl(
            &mut modules,
            &mut vec![package_name],
            src_path,
            &lib_oft_path,
        )?;

        self.0.insert(
            package_name,
            Package::Filesystem(meta, modules.into_iter().collect()),
        );

        Ok(())
    }

    /// Compiles a binary from a given module into a `flatanf::Program`.
    pub fn compile(
        self,
        root_package_name: Symbol,
        binary: &str,
    ) -> Result<Program, Error> {
        let mut builtins = HashMap::new();
        let mut root_meta = None;
        let mut mods = Vec::new();

        // Bundle up the packages.
        for (package_name, package) in self.0 {
            match package {
                Package::Builtins(mods) => {
                    builtins.insert(package_name, mods);
                }
                Package::Filesystem(meta, m) => {
                    if root_package_name == package_name {
                        root_meta = Some(meta);
                    }
                    mods.extend(m);
                }
            }
        }

        // Add the binary.
        let root_meta = root_meta.ok_or_else(|| {
            Error::NoSuchBinary(root_package_name, binary.to_string())
        })?;

        // Create the `flatanf::Program`.
        Program::from_modules(mods, builtins)
    }

    /// Loads metadata from the module in the given directory.
    pub fn load_metadata_from(
        &self,
        path: &Path,
    ) -> Result<PackageMetadata, Error> {
        let lits = parse_file(path.join("package.oftd"))?;
        PackageMetadata::from_literals(lits).map_err(Into::into)
    }

    /// Loads an `anf::Module` from the given path.
    pub fn load_module(path: &Path) -> Result<Module, Error> {
        let lits = parse_file(path)?;
        let ast_mod = ::ast::Module::from_values(lits)?;
        Module::convert(ast_mod)
    }
}
