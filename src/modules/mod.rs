//! The module system.

mod metadata;
#[cfg(test)]
mod tests;

use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt::Write;
use std::path::{Path, PathBuf};

use failure::ResultExt;
use symbol::Symbol;

use anf::Module;
use error::{Error, ErrorKind};
use flatanf::Program;
pub use modules::metadata::{BinaryComponentMetadata, ComponentsMetadata,
                            DependencyMetadata, LibraryComponentMetadata,
                            PackageMetadata};
use parser::parse_file;

#[derive(Clone, Debug)]
enum Package {
    Builtins(HashMap<Symbol, HashSet<Symbol>>),
    Filesystem(PathBuf, PackageMetadata, Vec<Module>),
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
    pub fn add_modules_from(&mut self, path: PathBuf) -> Result<(), Error> {
        let root_meta = self.load_metadata_from(&path)?;
        self.add_package_from(root_meta.name, path)
    }

    /// Loads a package from the given directory, without loading dependencies.
    /// However, this will panic if the dependencies are not already met.
    ///
    /// This is useful for loading, for example, the standard library.
    pub fn add_package_from(
        &mut self,
        package_name: Symbol,
        path: PathBuf,
    ) -> Result<(), Error> {
        let meta = self.load_metadata_from(&path)?;
        if package_name != meta.name {
            return Err(ErrorKind::MisnamedPackage(package_name, meta.name)
                .into());
        } else if meta.components.library.is_none() {
            return Err(ErrorKind::DependencyMustExportLib(package_name).into());
        }

        let mut modules = Vec::new();
        let src_path = path.join("src");
        let lib_oft_path = src_path.join("lib.oft");

        fn crawl(
            package_name: Symbol,
            modules: &mut Vec<Module>,
            mod_stack: &mut Vec<Symbol>,
            base: PathBuf,
            lib_oft_path: &Path,
        ) -> Result<(), Error> {
            for entry in base.read_dir().with_context(|_| {
                ErrorKind::CouldntReadPackageDir(
                    base.display().to_string(),
                    package_name,
                )
            })? {
                let entry = entry.with_context(|_| {
                    ErrorKind::CouldntReadPackageDir(
                        base.display().to_string(),
                        package_name,
                    )
                })?;
                let file_type = entry.file_type().with_context(|_| {
                    ErrorKind::CouldntReadPackageDir(
                        base.display().to_string(),
                        package_name,
                    )
                })?;
                if file_type.is_dir() {
                    let name = entry
                        .file_name()
                        .into_string()
                        .expect("Non-Unicode source directory name...")
                        .into();
                    mod_stack.push(name);
                    crawl(
                        package_name,
                        modules,
                        mod_stack,
                        entry.path(),
                        lib_oft_path,
                    )?;
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
                        return Err(ErrorKind::MisnamedModule(
                            name,
                            module.name,
                        ).into());
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
            package_name,
            &mut modules,
            &mut vec![package_name],
            src_path,
            &lib_oft_path,
        )?;

        self.0.insert(
            package_name,
            Package::Filesystem(path, meta, modules.into_iter().collect()),
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
        let mut root_meta_path = None;
        let mut mods = Vec::new();

        // Extract the prelude module's exports.
        let prelude = "std/prelude".into();
        let prelude_exports = self.exports_of("std".into(), prelude)?
            .into_iter()
            .map(|n| (prelude, n))
            .collect::<Vec<_>>();

        // Bundle up the packages.
        for (package_name, package) in self.0 {
            match package {
                Package::Builtins(mods) => {
                    //builtins.insert(package_name, mods);
                    unimplemented!();
                }
                Package::Filesystem(path, meta, ms) => {
                    if root_package_name == package_name {
                        root_meta_path = Some((meta, path));
                    }
                    mods.extend(ms.into_iter().map(|m| {
                        if m.name != prelude
                            && !m.imports.iter().any(|&(m, _)| m == prelude)
                        {
                            let mut m = m;
                            m.imports.extend(prelude_exports.iter().cloned());
                            m
                        } else {
                            m
                        }
                    }));
                }
            }
        }

        // Add the binary.
        let (root_meta, root_path) = match root_meta_path {
            Some((meta, path)) => (meta, path),
            None => {
                return Err(ErrorKind::NoSuchBinary(
                    root_package_name,
                    binary.to_string(),
                ).into());
            }
        };
        let binary_rel_path = root_meta
            .components
            .binaries
            .iter()
            .find(|bin| bin.name == binary)
            .map(|bin| &bin.path)
            .ok_or_else(|| {
                ErrorKind::NoSuchBinary(root_package_name, binary.to_string())
            })?;
        let binary_path = root_path.join(binary_rel_path);
        let binary = Packages::load_module(binary_path)?;
        mods.push(binary);

        // Create the `flatanf::Program`.
        Program::from_modules(mods, builtins)
    }

    /// Returns the exports of the given module.
    pub fn exports_of(
        &self,
        package: Symbol,
        module: Symbol,
    ) -> Result<BTreeSet<Symbol>, Error> {
        match self.0.get(&package) {
            Some(&Package::Builtins(ref package)) => {
                unimplemented!("{:?}", package);
            }
            Some(&Package::Filesystem(_, _, ref mods)) => {
                match mods.iter().find(|m| m.name == module) {
                    Some(m) => Ok(m.exports.clone()),
                    None => {
                        unimplemented!();
                    }
                }
            }
            None => {
                unimplemented!();
            }
        }
    }

    /// Loads metadata from the module in the given directory.
    pub fn load_metadata_from(
        &self,
        path: &Path,
    ) -> Result<PackageMetadata, Error> {
        let lits = parse_file(path.join("package.oftd"))?;
        PackageMetadata::from_literals(lits)
    }

    /// Loads an `anf::Module` from the given path.
    pub fn load_module<P: AsRef<Path>>(path: P) -> Result<Module, Error> {
        let lits = parse_file(path)?;
        let ast_mod = ::ast::Module::from_values(lits)?;
        Module::convert(ast_mod)
    }
}
