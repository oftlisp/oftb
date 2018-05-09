//! The intrinsics built into `oftb`.

#[macro_use]
mod macros;

use interpreter::{Store, Value};

fn boolify(b: bool) -> Value {
    if b {
        Value::Symbol("true".into())
    } else {
        Value::Nil
    }
}

fn print_values(store: &Store, values: &[Value]) {
    let mut first = true;
    for val in values {
        if first {
            first = false;
        } else {
            print!(" ");
        }
        print!("{}", val.display(store));
    }
}

intrinsics! {
    pkg "oftb-intrinsics" as Intrinsics;

    mod "" as root {
        fn eq[_store, _konts](a, b) {
            boolify(a == b)
        }

        fn equals[store, _konts](a, b) {
            boolify(a.equals(b, store))
        }

        fn list[store, _konts](*args) {
            let mut l = Value::Nil;
            for &x in args.iter().rev() {
                let head = store.store(x);
                let tail = store.store(l);
                l = Value::Cons(head, tail);
            }
            l
        }
    }

    mod "io" as io {
        fn print[store, _konts](*args) {
            print_values(store, args);
            Value::Nil
        }

        fn println[store, _konts](*args) {
            print_values(store, args);
            println!();
            Value::Nil
        }
    }

    mod "math" as math {
        fn add[_store, _konts](l, r) {
            match (l, r) {
                (Value::Byte(l), Value::Byte(r)) => Value::Byte(l + r),
                (Value::Fixnum(l), Value::Fixnum(r)) => Value::Fixnum(l + r),
                _ => panic!("TODO Math Type Error {:?} {:?}", l, r),
            }
        }

        fn mul[_store, _konts](l, r) {
            match (l, r) {
                (Value::Byte(l), Value::Byte(r)) => Value::Byte(l * r),
                (Value::Fixnum(l), Value::Fixnum(r)) => Value::Fixnum(l * r),
                _ => panic!("TODO Math Type Error {:?} {:?}", l, r),
            }
        }

        fn sub[_store, _konts](l, r) {
            match (l, r) {
                (Value::Byte(l), Value::Byte(r)) => Value::Byte(l - r),
                (Value::Fixnum(l), Value::Fixnum(r)) => Value::Fixnum(l - r),
                _ => panic!("TODO Math Type Error {:?} {:?}", l, r),
            }
        }
    }
}
