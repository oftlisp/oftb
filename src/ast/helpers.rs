use symbol::Symbol;

use literal::Literal;

/// Tries to convert a literal to a `module` statement.
pub fn convert_module(
    lit: &Literal,
) -> Option<(Symbol, Vec<Symbol>, Vec<(Symbol, Option<Literal>)>)> {
    let (hd, mut tl) = lit.as_shl()?;
    if hd.as_str() != "module" {
        return None;
    }
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

    Some((name, exports, attrs))
}

/// Tries to convert a literal to an `import` statement.
pub fn convert_import(lit: &Literal) -> Option<(Symbol, Vec<Symbol>)> {
    let mut l = lit.as_symbol_list()?;
    if l.len() < 2 || l[0].as_str() != "import" {
        return None;
    }

    let s = l.drain(0..2).last().unwrap();
    Some((s, l))
}

/// Checks if a literal is an `import` statement.
pub fn is_import(lit: &Literal) -> bool {
    if let Some(syms) = lit.as_symbol_list() {
        if syms.len() >= 2 {
            syms[0].as_str() == "import"
        } else {
            false
        }
    } else {
        false
    }
}
