//! The intrinsics built into `oftb`.

#[macro_use]
mod macros;

use interpreter::Value;

intrinsics! {
    mod "oftb-intrinsics";

    fn apply[store, konts](func args) {
        warn!("TODO apply");
        warn!("{:?}", func);
        warn!("{:?}", args);
        Value::Nil
    }
}
