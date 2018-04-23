use std::collections::BTreeMap;

use semver::{Version, VersionReq};
use symbol::Symbol;

use error::{Error, ErrorKind};
use literal::Literal;

/// Package metadata, stored in `package.oftd` files.
#[derive(Clone, Debug, Hash, PartialEq, PartialOrd)]
pub struct PackageMetadata {
    /// The authors of the package.
    pub authors: Vec<String>,

    /// The package's components.
    pub components: ComponentsMetadata,

    /// The package's dependencies.
    pub dependencies: BTreeMap<Symbol, DependencyMetadata>,

    /// The SPDX identifier for the license the package is under.
    ///
    /// For example,
    ///
    ///  - `Apache-2.0`
    ///  - `GPL-3.0-or-later`
    ///  - `LGPL-3.0-or-later`
    ///  - `MIT`
    pub license: Option<String>,

    /// The name of the package. Required.
    pub name: Symbol,

    /// The version of the package. Required.
    pub version: Version,
}

impl PackageMetadata {
    /// Converts from `Literal`s.
    pub fn from_literals(lits: Vec<Literal>) -> Result<PackageMetadata, Error> {
        let mut authors = None;
        let mut components = None;
        let mut dependencies = None;
        let mut license = None;
        let mut name = None;
        let mut version = None;

        for lit in lits {
            let (s, l) = if let Some(val) = lit.as_shl() {
                val
            } else {
                return Err(ErrorKind::Unexpected("a metadata item", lit).into());
            };
            match s.as_str() {
                "authors" => {
                    if authors.is_some() {
                        return Err(ErrorKind::DuplicateField(s).into());
                    }

                    let v = l.into_iter()
                        .map(|lit| match lit {
                            Literal::String(s) => Ok(s),
                            _ => Err(Error::from(ErrorKind::Unexpected(
                                "an author",
                                lit,
                            ))),
                        })
                        .collect::<Result<_, _>>()?;
                    authors = Some(v);
                }
                "components" => {
                    if components.is_some() {
                        return Err(ErrorKind::DuplicateField(s).into());
                    }

                    let v = ComponentsMetadata::from_literals(l)?;
                    components = Some(v);
                }
                "dependencies" => {
                    if dependencies.is_some() {
                        return Err(ErrorKind::DuplicateField(s).into());
                    }

                    let deps = l.into_iter()
                        .map(|lit| {
                            if let Some((name, lits)) = lit.as_shl() {
                                Ok((
                                    name,
                                    DependencyMetadata::from_literals(lits)?,
                                ))
                            } else {
                                Err(Error::from(ErrorKind::Unexpected(
                                    "a dependency",
                                    lit,
                                )))
                            }
                        })
                        .collect::<Result<_, _>>()?;
                    dependencies = Some(deps);
                }
                "license" => {
                    if license.is_some() {
                        return Err(ErrorKind::DuplicateField(s).into());
                    }

                    if l.len() != 1 {
                        return Err(ErrorKind::Unexpected(
                            "a license",
                            Literal::list(l),
                        ).into());
                    }
                    let l = l.into_iter().next().unwrap();
                    if let Literal::String(l) = l {
                        license = Some(l);
                    } else {
                        return Err(ErrorKind::Unexpected("a license", l).into());
                    }
                }
                "name" => {
                    if name.is_some() {
                        return Err(ErrorKind::DuplicateField(s).into());
                    }

                    if l.len() != 1 {
                        return Err(ErrorKind::Unexpected(
                            "the package name",
                            Literal::list(l),
                        ).into());
                    }
                    let n = l.into_iter().next().unwrap();
                    if let Literal::Symbol(n) = n {
                        name = Some(n);
                    } else {
                        return Err(ErrorKind::Unexpected(
                            "the package name",
                            n,
                        ).into());
                    }
                }
                "version" => {
                    if version.is_some() {
                        return Err(ErrorKind::DuplicateField(s).into());
                    }

                    if l.len() != 1 {
                        return Err(ErrorKind::Unexpected(
                            "the package version",
                            Literal::list(l),
                        ).into());
                    }
                    let v = l.into_iter().next().unwrap();
                    if let Literal::String(v) = v {
                        let v = v.parse()
                            .map_err(ErrorKind::IllegalPackageVersion)?;
                        version = Some(v);
                    } else {
                        return Err(ErrorKind::Unexpected(
                            "the package version",
                            v,
                        ).into());
                    }
                }
                _ => {
                    return Err(ErrorKind::Unexpected("a metadata item", lit).into())
                }
            }
        }

        Ok(PackageMetadata {
            authors: authors.unwrap_or_default(),
            components: components
                .ok_or(ErrorKind::MissingField("components".into()))?,
            dependencies: dependencies.unwrap_or_default(),
            license,
            name: name.ok_or(ErrorKind::MissingField("name".into()))?,
            version: version.ok_or(ErrorKind::MissingField("version".into()))?,
        })
    }

    /// Converts to `Literal`s.
    pub fn to_literals(self) -> Vec<Literal> {
        let mut l = Vec::new();
        if self.authors.len() != 0 {
            let mut authors = vec![Literal::Symbol("authors".into())];
            authors.extend(self.authors.into_iter().map(Literal::String));
            l.push(Literal::list(authors));
        }
        if let Some(license) = self.license {
            l.push(Literal::list(vec![
                Literal::Symbol("license".into()),
                Literal::String(license),
            ]));
        }
        l.push(Literal::list(vec![
            Literal::Symbol("name".into()),
            Literal::Symbol(self.name.into()),
        ]));
        l.push(Literal::list(vec![
            Literal::Symbol("version".into()),
            Literal::String(self.version.to_string()),
        ]));

        l.push(Literal::Cons(
            Box::new(Literal::Symbol("components".into())),
            Box::new(self.components.to_literal()),
        ));

        if self.dependencies.len() != 0 {
            let mut deps = self.dependencies.into_iter().collect::<Vec<_>>();
            deps.sort_by_key(|&(n, _)| n.as_str());
            let mut dep_lit = vec![Literal::Symbol("dependencies".into())];
            dep_lit.extend(deps.into_iter().map(|(name, dep)| {
                Literal::Cons(
                    Box::new(Literal::Symbol(name.into())),
                    Box::new(dep.to_literal()),
                )
            }));
            l.push(Literal::list(dep_lit));
        }

        l
    }
}

/// The metadata associated with the `components` field of `PackageMetadata`.
#[derive(Clone, Debug, Hash, PartialEq, PartialOrd)]
pub struct ComponentsMetadata {
    /// The library component, if one exists.
    pub library: Option<LibraryComponentMetadata>,

    /// The binary components, if any, associated with the package.
    pub binaries: Vec<BinaryComponentMetadata>,
}

impl ComponentsMetadata {
    /// Converts from `Literal`s.
    pub fn from_literals(
        lits: Vec<Literal>,
    ) -> Result<ComponentsMetadata, Error> {
        let mut library = None;
        let mut binaries = Vec::new();

        for lit in lits {
            let (s, l) = if let Some(val) = lit.as_shl() {
                val
            } else {
                return Err(ErrorKind::Unexpected("a component", lit).into());
            };
            match s.as_str() {
                "library" => {
                    if library.is_some() {
                        return Err(ErrorKind::DuplicateField(s).into());
                    }

                    let lib = LibraryComponentMetadata::from_literals(l)?;
                    library = Some(lib);
                }
                "binary" => {
                    let bin = BinaryComponentMetadata::from_literals(l)?;
                    binaries.push(bin);
                }
                _ => {
                    return Err(ErrorKind::Unexpected("a component", lit).into())
                }
            }
        }

        Ok(ComponentsMetadata { library, binaries })
    }

    /// Converts to a `Literal`.
    pub fn to_literal(self) -> Literal {
        let mut l = Vec::new();
        if let Some(library) = self.library {
            l.push(Literal::Cons(
                Box::new(Literal::Symbol("library".into())),
                Box::new(library.to_literal()),
            ));
        }
        l.extend(self.binaries.into_iter().map(|b| {
            Literal::Cons(
                Box::new(Literal::Symbol("binary".into())),
                Box::new(b.to_literal()),
            )
        }));
        Literal::list(l)
    }
}

/// The metadata associated with the `components.library` field of
/// `PackageMetadata`.
///
/// Since a library (currently) has no metadata other than that of the package
/// itself, this is a unit struct.
#[derive(Clone, Debug, Hash, PartialEq, PartialOrd)]
pub struct LibraryComponentMetadata;

impl LibraryComponentMetadata {
    /// Converts from a `Literal`.
    pub fn from_literals(
        mut lits: Vec<Literal>,
    ) -> Result<LibraryComponentMetadata, Error> {
        if let Some(lit) = lits.pop() {
            Err(ErrorKind::Unexpected("a library component", lit).into())
        } else {
            Ok(LibraryComponentMetadata)
        }
    }

    /// Converts to a `Literal`.
    pub fn to_literal(self) -> Literal {
        Literal::Nil
    }
}

/// The metadata associated with the `components.binaries` field of
/// `PackageMetadata`.
#[derive(Clone, Debug, Hash, PartialEq, PartialOrd)]
pub struct BinaryComponentMetadata {
    /// The name of the binary. Required.
    pub name: String,

    /// The path of the main file of the binary. Required.
    pub path: String,
}

impl BinaryComponentMetadata {
    /// Converts from a `Literal`.
    pub fn from_literals(
        lits: Vec<Literal>,
    ) -> Result<BinaryComponentMetadata, Error> {
        let mut name = None;
        let mut path = None;

        for lit in lits {
            let (s, lit) = if let Some(val) = lit.as_shp() {
                val
            } else {
                return Err(ErrorKind::Unexpected(
                    "a binary component metadata item",
                    lit,
                ).into());
            };
            match s.as_str() {
                "name" => {
                    if name.is_some() {
                        return Err(ErrorKind::DuplicateField(s).into());
                    } else if let Literal::String(n) = lit {
                        name = Some(n);
                    } else {
                        return Err(ErrorKind::Unexpected("a binary name", lit).into());
                    }
                }
                "path" => {
                    if path.is_some() {
                        return Err(ErrorKind::DuplicateField(s).into());
                    } else if let Literal::String(p) = lit {
                        path = Some(p);
                    } else {
                        return Err(ErrorKind::Unexpected("a path", lit).into());
                    }
                }
                _ => {
                    return Err(ErrorKind::Unexpected(
                        "a binary component metadata item",
                        lit,
                    ).into())
                }
            }
        }

        Ok(BinaryComponentMetadata {
            name: name.ok_or(ErrorKind::MissingField("name".into()))?,
            path: path.ok_or(ErrorKind::MissingField("path".into()))?,
        })
    }

    /// Converts to a `Literal`.
    pub fn to_literal(self) -> Literal {
        Literal::list(vec![
            Literal::list(vec![
                Literal::Symbol("name".into()),
                Literal::String(self.name),
            ]),
            Literal::list(vec![
                Literal::Symbol("path".into()),
                Literal::String(self.path),
            ]),
        ])
    }
}

/// The metadata associated with the `dependencies` field of `PackageMetadata`.
#[derive(Clone, Debug, Hash, PartialEq, PartialOrd)]
pub struct DependencyMetadata {
    /// The Git repo to get the dependency from, if any.
    pub git: Option<String>,

    /// The required version of the dependency.
    pub version: VersionReq,
}

impl DependencyMetadata {
    /// Converts from `Literal`s.
    pub fn from_literals(
        lits: Vec<Literal>,
    ) -> Result<DependencyMetadata, Error> {
        let mut git = None;
        let mut version = None;

        for lit in lits {
            let (s, lit) = if let Some(val) = lit.as_shp() {
                val
            } else {
                return Err(ErrorKind::Unexpected(
                    "a binary component metadata item",
                    lit,
                ).into());
            };
            match s.as_str() {
                "git" => {
                    if git.is_some() {
                        return Err(ErrorKind::DuplicateField(s).into());
                    } else if let Literal::String(url) = lit {
                        git = Some(url);
                    } else {
                        return Err(ErrorKind::Unexpected(
                            "a Git clone URL",
                            lit,
                        ).into());
                    }
                }
                "version" => {
                    if version.is_some() {
                        return Err(ErrorKind::DuplicateField(s).into());
                    } else if let Literal::String(v) = lit {
                        let v = v.parse()
                            .map_err(ErrorKind::IllegalDependencyVersion)?;
                        version = Some(v);
                    } else {
                        return Err(ErrorKind::Unexpected(
                            "a dependency version",
                            lit,
                        ).into());
                    }
                }
                _ => {
                    return Err(ErrorKind::Unexpected(
                        "a dependency metadata item",
                        lit,
                    ).into())
                }
            }
        }

        Ok(DependencyMetadata {
            git,
            version: version.ok_or(ErrorKind::MissingField("version".into()))?,
        })
    }

    /// Converts to a `Literal`.
    pub fn to_literal(self) -> Literal {
        let mut l = Vec::new();
        if let Some(git) = self.git {
            l.push(Literal::list(vec![
                Literal::Symbol("git".into()),
                Literal::String(git),
            ]));
        }
        l.push(Literal::list(vec![
            Literal::Symbol("version".into()),
            Literal::String(self.version.to_string()),
        ]));
        Literal::list(l)
    }
}
