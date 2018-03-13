use std::sync::atomic::{AtomicUsize, Ordering};

use symbol::Symbol;

static COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Generates a new symbol.
pub fn gensym() -> Symbol {
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    assert_ne!(n, ::std::usize::MAX);
    Symbol::from(format!("gensym@{}", n))
}
