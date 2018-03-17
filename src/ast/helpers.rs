use symbol::Symbol;

use literal::Literal;

/// Tries to convert a literal to a statement similar to a `module` or `import`
/// statement.
pub fn convert_moduleish(
    head: Symbol,
    lit: &Literal,
) -> Option<(Symbol, Vec<Symbol>)> {
    let mut l = lit.as_symbol_list()?;
    if l.len() < 2 || l[0] != head {
        return None;
    }

    let s = l.drain(0..2).last().unwrap();
    Some((s, l))
}

/// Checks if a literal is a statement similar to a `module` or `import`
/// statement.
pub fn is_moduleish(head: Symbol, lit: &Literal) -> bool {
    if let Some(syms) = lit.as_symbol_list() {
        if syms.len() >= 2 {
            syms[0] == head
        } else {
            false
        }
    } else {
        false
    }
}
