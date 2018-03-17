use parser::parse_program;
use modules::{BinaryComponentMetadata, ComponentsMetadata, DependencyMetadata,
              LibraryComponentMetadata, PackageMetadata};

#[test]
fn parses_blogpost_package_oftd() {
    let src = r#"(authors
  "Nathan Ringo <remexre@gmail.com>")
(license "MIT")
(name foo)
(version "0.1.0")

(components
  (library)
  (binary
    (name "foo")
    (path "src/main.oft")))

(dependencies
  (combinargs
    (version "^0.2.1"))
  (grid
    (git "https://github.com/remexre/oftlisp-grid.git")
    (version "^0.1.0")))
"#;
    let data = PackageMetadata {
        authors: vec!["Nathan Ringo <remexre@gmail.com>".to_string()],
        components: ComponentsMetadata {
            library: Some(LibraryComponentMetadata),
            binaries: vec![
                BinaryComponentMetadata {
                    name: "foo".to_string(),
                    path: "src/main.oft".to_string(),
                },
            ],
        },
        dependencies: vec![
            (
                "combinargs".into(),
                DependencyMetadata {
                    git: None,
                    version: "0.2.1".parse().unwrap(),
                },
            ),
            (
                "grid".into(),
                DependencyMetadata {
                    git: Some(
                        "https://github.com/remexre/oftlisp-grid.git"
                            .to_string(),
                    ),
                    version: "0.1.0".parse().unwrap(),
                },
            ),
        ].into_iter()
            .collect(),
        license: Some("MIT".to_string()),
        name: "foo".into(),
        version: "0.1.0".parse().unwrap(),
    };

    let lits = parse_program(src).unwrap();
    assert_eq!(data.clone().to_literals(), lits);
    let meta = PackageMetadata::from_literals(lits).unwrap();
    assert_eq!(meta, data);
}
