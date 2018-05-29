use symbol::Symbol;

use literal::Literal;

/// Tries to convert a literal to a `module` or `import` statement.
pub fn convert_modulish(
    lit: &Literal,
) -> Option<(
    Symbol,
    Symbol,
    Vec<Symbol>,
    Vec<(Symbol, Option<Literal>)>,
)> {
    let (hd, mut tl) = lit.as_shl()?;
    tl.reverse();

    let name = if let Some(Literal::Symbol(s)) = tl.pop() {
        s
    } else {
        return None;
    };

    let exports = if let Some(Literal::Vector(e)) = tl.pop() {
        e.into_iter()
            .map(|l| match l {
                Literal::Symbol(s) => Some(s),
                _ => None,
            })
            .collect::<Option<_>>()?
    } else {
        return None;
    };

    let mut attrs = Vec::new();
    for val in tl {
        attrs.push(match val {
            Literal::Symbol(s) => (s, None),
            Literal::Cons(h, t) => match *h {
                Literal::Symbol(h) => (h, Some(*t)),
                _ => return None,
            },
            _ => return None,
        });
    }

    Some((hd, name, exports, attrs))
}

/// Tries to convert a literal to a `module` statement.
pub fn convert_module(
    lit: &Literal,
) -> Option<(
    Symbol,
    Vec<Symbol>,
    Vec<(Symbol, Option<Literal>)>,
)> {
    convert_modulish(lit).and_then(|(hd, name, exports, attrs)| {
        if hd.as_str() == "module" {
            Some((name, exports, attrs))
        } else {
            None
        }
    })
}

/// Tries to convert a literal to an `import` statement.
pub fn convert_import(lit: &Literal) -> Option<(Symbol, Vec<Symbol>)> {
    convert_modulish(lit).and_then(|(hd, name, imports, attrs)| {
        if hd.as_str() == "import" && attrs.len() == 0 {
            Some((name, imports))
        } else {
            None
        }
    })
}

/// Checks if a literal is an `import` statement.
pub fn is_import(lit: &Literal) -> bool {
    convert_import(lit).is_some()
}
