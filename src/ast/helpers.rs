use symbol::Symbol;

use ast::Literal;

/// Converts a list of symbols to a `Vec<Symbol>`.
fn as_symbol_list(mut lit: &Literal) -> Option<Vec<Symbol>> {
    let mut out = Vec::new();
    loop {
        match *lit {
            Literal::Cons(ref h, ref t) => if let Literal::Symbol(s) = **h {
                out.push(s);
                lit = &*t;
            } else {
                return None;
            },
            Literal::Nil => return Some(out),
            _ => return None,
        }
    }
}

/// Tries to convert a literal to a statement similar to a `module` or `import`
/// statement.
pub fn convert_moduleish(
    head: Symbol,
    lit: &Literal,
) -> Option<(Symbol, Vec<Symbol>)> {
    let mut l = as_symbol_list(lit)?;
    if l.len() < 2 || l[0] != head {
        return None;
    }

    let s = l.drain(0..2).last().unwrap();
    Some((s, l))
}

/// Checks if a literal is a statement similar to a `module` or `import`
/// statement.
pub fn is_moduleish(head: Symbol, lit: &Literal) -> bool {
    if let Some(syms) = as_symbol_list(lit) {
        if syms.len() >= 2 {
            syms[0] == head
        } else {
            false
        }
    } else {
        false
    }
}

/// Returns the list this literal contains, or `None` if it does not contain
/// one.
pub fn as_list(mut lit: &Literal) -> Option<Vec<Literal>> {
    let mut l = Vec::new();
    loop {
        match *lit {
            Literal::Cons(ref h, ref t) => {
                l.push((**h).clone());
                lit = &*t;
            }
            Literal::Nil => return Some(l),
            _ => return None,
        }
    }
}

/// Checks if a literal represents a proper list.
pub fn is_list(mut lit: &Literal) -> bool {
    loop {
        match *lit {
            Literal::Cons(_, ref t) => lit = &*t,
            Literal::Nil => return true,
            _ => return false,
        }
    }
}

/// Checks if a literal represents a list whose head is the given symbol.
pub fn is_shl(sym: Symbol, lit: &Literal) -> bool {
    if let Literal::Cons(ref h, ref t) = *lit {
        Literal::Symbol(sym) == **h && is_list(t)
    } else {
        false
    }
}
