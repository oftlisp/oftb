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

fn print_values(store: &Store, values: &[Value], printlike: bool) {
    let mut first = true;
    for val in values {
        if first {
            first = false;
        } else {
            print!(" ");
        }
        print!("{}", val.display(store, printlike));
    }
}

intrinsics! {
    pkg "intrinsics" as Intrinsics;

    mod "" as root {
        fn car[store, _k](l) {
            match l {
                Value::Cons(h, _) => store.get(h),
                _ => unimplemented!("Can't take car of {:?}", l)
            }
        }

        fn cdr[store, _k](l) {
            match l {
                Value::Cons(_, t) => store.get(t),
                _ => unimplemented!("Can't take cdr of {:?}", l)
            }
        }

        fn cons[store, _k](h, t) {
            let h = store.store(h);
            let t = store.store(t);
            Value::Cons(h, t)
        }

        fn eq[_s, _k](a, b) {
            boolify(a == b)
        }

        fn equals[store, _k](a, b) {
            boolify(a.equals(b, store))
        }

        fn list[store, _k](*args) {
            let mut l = Value::Nil;
            for &x in args.iter().rev() {
                let head = store.store(x);
                let tail = store.store(l);
                l = Value::Cons(head, tail);
            }
            l
        }

        fn panic[store, _k](msg) {
            panic!("{}", msg.display(store, true))
        }
    }

    mod "io" as io {
        fn print[store, _k](*args) {
            print_values(store, args, true);
            Value::Nil
        }

        fn println[store, _k](*args) {
            print_values(store, args, true);
            println!();
            Value::Nil
        }

        fn write[store, _k](*args) {
            print_values(store, args, false);
            Value::Nil
        }

        fn writeln[store, _k](*args) {
            print_values(store, args, false);
            println!();
            Value::Nil
        }

        fn write_bytes[store, _k](bytes) {
            use std::io::{stdout,Write};

            let bytes = if let Value::Bytes(addr, len) = bytes {
                store.get_bytes(addr, len)
            } else {
                unimplemented!("TODO Type Error")
            };
            stdout().write_all(bytes).unwrap();
            Value::Nil
        }
    }

    mod "oftb" as oftb {
        fn read_dir[store, _k](path) {
            let path = if let Value::String(addr, len) = path {
                store.get_str(addr, len)
            } else {
                unimplemented!("TODO Type Error")
            };

            unimplemented!("read_dir {:?}", path)
        }

        fn read_file[store, _k](path) {
            let path = if let Value::String(addr, len) = path {
                store.get_str(addr, len)
            } else {
                unimplemented!("TODO Type Error")
            };

            unimplemented!("read_file {:?}", path)
        }
    }

    mod "math" as math {
        fn add[_s, _k](l, r) {
            match (l, r) {
                (Value::Byte(l), Value::Byte(r)) => Value::Byte(l + r),
                (Value::Fixnum(l), Value::Fixnum(r)) => Value::Fixnum(l + r),
                _ => panic!("TODO Math Type Error {:?} {:?}", l, r),
            }
        }

        fn mul[_s, _k](l, r) {
            match (l, r) {
                (Value::Byte(l), Value::Byte(r)) => Value::Byte(l * r),
                (Value::Fixnum(l), Value::Fixnum(r)) => Value::Fixnum(l * r),
                _ => panic!("TODO Math Type Error {:?} {:?}", l, r),
            }
        }

        fn sub[_s, _k](l, r) {
            match (l, r) {
                (Value::Byte(l), Value::Byte(r)) => Value::Byte(l - r),
                (Value::Fixnum(l), Value::Fixnum(r)) => Value::Fixnum(l - r),
                _ => panic!("TODO Math Type Error {:?} {:?}", l, r),
            }
        }
    }

    mod "types" as types {
        fn is_byte    [_s, _k](x) { boolify(match x {
            Value::Byte(_)      => true, _ => false }) }
        fn is_bytes   [_s, _k](x) { boolify(match x {
            Value::Bytes(_, _)  => true, _ => false }) }
        fn is_cons    [_s, _k](x) { boolify(match x {
            Value::Cons(_, _)   => true, _ => false }) }
        fn is_fixnum  [_s, _k](x) { boolify(match x {
            Value::Fixnum(_)    => true, _ => false }) }
        fn is_function[_s, _k](x) { boolify(match x {
            Value::Closure(_) => true, Value::Intrinsic(_) => true,
            _ => false }) }
        fn is_nil     [_s, _k](x) { boolify(match x {
            Value::Nil          => true, _ => false }) }
        fn is_string  [_s, _k](x) { boolify(match x {
            Value::String(_, _) => true, _ => false }) }
        fn is_symbol  [_s, _k](x) { boolify(match x {
            Value::Symbol(_)    => true, _ => false }) }
        fn is_vector  [_s, _k](x) { boolify(match x {
            Value::Vector(_, _) => true, _ => false }) }
    }
}
