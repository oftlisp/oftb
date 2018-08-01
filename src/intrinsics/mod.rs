//! The intrinsics built into `oftb`.

#[macro_use]
mod macros;

use std::cmp::Ordering;
use std::process::exit;

use interpreter::{Addr, Kont, Store, Value};
use {parse_file, Literal};

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
        } else if printlike {
            print!(" ");
        }
        print!("{}", val.display(store, printlike));
    }
}

intrinsics! {
    pkg "intrinsics" as Intrinsics;

    mod "" as root {
        fn apply[store, konts](func, args) {
            let mut lst = args;
            let mut args = Vec::new();
            while let Value::Cons(hd, tl) = lst {
                args.push(store.get(hd));
                lst = store.get(tl);
            }
            if lst == Value::Nil {
                return ::interpreter::eval::apply(func, args, store, konts)
            } else {
                panic!("type error in apply (not a list)")
            }
        }

        fn car[store, _k](l) {
            match l {
                Value::Cons(h, _) => store.get(h),
                _ => unimplemented!("Can't take car of {}", l.display(store, false))
            }
        }

        fn cdr[store, _k](l) {
            match l {
                Value::Cons(_, t) => store.get(t),
                _ => unimplemented!("Can't take cdr of {}", l.display(store, false))
            }
        }

        fn compare[store, _k](a, b) {
            match a.compare(b, store) {
                Ordering::Greater => Value::Symbol("gt".into()),
                Ordering::Equal => Value::Symbol("eq".into()),
                Ordering::Less => Value::Symbol("lt".into()),
            }
        }

        fn cons[store, _k](h, t) {
            let h = store.store(h);
            let t = store.store(t);
            Value::Cons(h, t)
        }

        fn eq_num[store, _k](l, r) {
            boolify(match (l, r) {
                (Value::Byte(l), Value::Byte(r)) => l == r,
                (Value::Byte(l), Value::Fixnum(r)) => l as isize == r,
                (Value::Fixnum(l), Value::Byte(r)) => l == r as isize,
                (Value::Fixnum(l), Value::Fixnum(r)) => l == r,
                _ => panic!(
                    "type error in (eq_num {} {})",
                    l.display(store, false),
                    r.display(store, false),
                ),
            })
        }

        fn eq[_s, _k](a, b) {
            boolify(a == b)
        }

        fn equals[store, _k](a, b) {
            boolify(a.equals(b, store))
        }

        fn gensym[_s, _k]() {
            Value::Symbol(::gensym::gensym())
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
            if let Value::Cons(hd, tl) = msg {
                if store.get(hd) == Value::Symbol("exit".into()) {
                    if let Value::Fixnum(n) = store.get(tl) {
                        // TODO: When this gets more Erlang-like, the behavior here will need to
                        // change.
                        exit(n as i32);
                    }
                }
            } else if msg == Value::Symbol("exit".into()) {
                exit(0);
            }
            panic!("{}", msg.display(store, false))
        }
    }

    mod "byte" as byte {
        fn and[_s, _k](a, b) {
            typeck_name!(a as Value::Byte(a), b as Value::Byte(b));
            Value::Byte(a & b)
        }

        fn not[_s, _k](n) {
            typeck_name!(n as Value::Byte(n));
            Value::Byte(!n)
        }

        fn or[_s, _k](a, b) {
            typeck_name!(a as Value::Byte(a), b as Value::Byte(b));
            Value::Byte(a | b)
        }

        fn rol[_s, _k](a, b) {
            typeck_name!(a as Value::Byte(a), b as Value::Byte(b));
            Value::Byte(a.rotate_left(b as u32))
        }

        fn ror[_s, _k](a, b) {
            typeck_name!(a as Value::Byte(a), b as Value::Byte(b));
            Value::Byte(a.rotate_right(b as u32))
        }

        fn shl[_s, _k](a, b) {
            typeck_name!(a as Value::Byte(a), b as Value::Byte(b));
            Value::Byte(a.checked_shl(b as u32).unwrap_or(0))
        }

        fn shr[_s, _k](a, b) {
            typeck_name!(a as Value::Byte(a), b as Value::Byte(b));
            Value::Byte(a.checked_shr(b as u32).unwrap_or(0))
        }

        fn xor[_s, _k](a, b) {
            typeck_name!(a as Value::Byte(a), b as Value::Byte(b));
            Value::Byte(a ^ b)
        }
    }

    mod "bytes" as bytes {
        fn append[store, _k](l, r) {
            // TODO: Make this more optimized.
            match (l, r) {
                (Value::Bytes(la, ll), Value::Bytes(ra, rl)) => {
                    let mut vals = store.get_bytes(la, ll).to_owned();
                    vals.extend(store.get_bytes(ra, rl));
                    let (a, l) = store.store_bytes(&vals);
                    Value::Bytes(a, l)
                }
                _ => unimplemented!("TODO type error in bytes-append")
            }
        }

        fn length[_s, _k](s) {
            if let Value::Bytes(_, l) = s {
                Value::Fixnum(l as isize)
            } else {
                unimplemented!("TODO Type Error in bytes-length")
            }
        }

        fn nth[store, _k](n, s) {
            if let (Value::Fixnum(n), Value::Bytes(a, l)) = (n, s) {
                let n = n as usize;
                if n < l {
                    let a: usize = a.into();
                    store.get_vec_val(Addr::from(a + n))
                } else {
                    unimplemented!("TODO out of bounds in bytes-nth")
                }
            } else {
                unimplemented!("TODO Type Error in bytes-nth")
            }
        }

        fn slice[_s, _k](start, end, s) {
            let (a, l) = if let (
                Value::Fixnum(start),
                Value::Fixnum(end),
                Value::Bytes(a, l),
            ) = (start, end, s)
            {
                let start = start as usize;
                let end = end as usize;
                if start <= l && end <= l && start <= end {
                    let a: usize = a.into();
                    (a + start as usize, end - start)
                } else {
                    unimplemented!(
                        "TODO out of bounds in bytes-slice (bounds of [{}, {}) on bytes of length {})",
                        start,
                        end,
                        l,
                    )
                }
            } else {
                unimplemented!("TODO Type Error in bytes-slice")
            };

            Value::Bytes(a.into(), l)
        }
    }

    mod "convert" as convert {
        fn byte_to_bytes[store, _k](b) {
            if let Value::Byte(b) = b {
                let (a, l) = store.store_bytes(&[b]);
                Value::Bytes(a, l)
            } else {
                unimplemented!("TODO Type Error in byte->bytes")
            }
        }

        fn byte_to_fixnum[_s, _k](b) {
            typeck_name!(b as Value::Byte(b));
            Value::Fixnum(b as isize)
        }

        fn fixnum_to_byte[_s, _k](n) {
            typeck_name!(n as Value::Fixnum(n));
            Value::Byte(n as u8)
        }

        fn list_to_vector[store, _k](s) {
            let mut lst = s;
            let mut vec = Vec::new();
            while let Value::Cons(hd, tl) = lst {
                vec.push(hd);
                lst = store.get(tl);
            }
            if lst == Value::Nil {
                let (a, l) = store.store_vec(&vec);
                Value::Vector(a, l)
            } else {
                unimplemented!("TODO Type Error in list->vector")
            }
        }

        fn string_to_bytes[store, _k](s) {
            typeck_name!(s as Value::String(a, l));
            let s = store.get_str(s.0, s.1).to_string();
            let (a, l) = store.store_bytes(s.as_bytes());
            Value::Bytes(a, l)
        }

        fn string_to_symbol[store, _k](s) {
            if let Value::String(a, l) = s {
                // TODO Check the string for validity.
                Value::Symbol(store.get_str(a, l).into())
            } else {
                unimplemented!("TODO Type Error in string->symbol: {}", s.display(store, false))
            }
        }

        fn symbol_to_string[store, _k](s) {
            if let Value::Symbol(s) = s {
                let (a, l) = store.store_str(s.as_str());
                Value::String(a, l)
            } else {
                unimplemented!("TODO Type Error in symbol->string: {}", s.display(store, false))
            }
        }
    }

    mod "fixnum" as fixnum {
        fn and[_s, _k](a, b) {
            typeck_name!(a as Value::Fixnum(a), b as Value::Fixnum(b));
            Value::Fixnum(a & b)
        }

        fn not[_s, _k](n) {
            typeck_name!(n as Value::Fixnum(n));
            Value::Fixnum(!n)
        }

        fn or[_s, _k](a, b) {
            typeck_name!(a as Value::Fixnum(a), b as Value::Fixnum(b));
            Value::Fixnum(a | b)
        }

        fn rol[_s, _k](a, b) {
            typeck_name!(a as Value::Fixnum(a), b as Value::Fixnum(b));
            Value::Fixnum(a.rotate_left(b as u32))
        }

        fn ror[_s, _k](a, b) {
            typeck_name!(a as Value::Fixnum(a), b as Value::Fixnum(b));
            Value::Fixnum(a.rotate_right(b as u32))
        }

        fn shl[_s, _k](a, b) {
            typeck_name!(a as Value::Fixnum(a), b as Value::Fixnum(b));
            Value::Fixnum(a.checked_shl(b as u32).unwrap_or(0))
        }

        fn shr[_s, _k](a, b) {
            typeck_name!(a as Value::Fixnum(a), b as Value::Fixnum(b));
            Value::Fixnum(a.checked_shr(b as u32).unwrap_or(0))
        }

        fn xor[_s, _k](a, b) {
            typeck_name!(a as Value::Fixnum(a), b as Value::Fixnum(b));
            Value::Fixnum(a ^ b)
        }
    }

    mod "io" as io {
        fn debug_print[store, _k](value) {
            eprintln!("{}", value.display(store, false));
            Value::Nil
        }

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
            use std::io::{stdout, Write};

            let bytes = if let Value::Bytes(addr, len) = bytes {
                store.get_bytes(addr, len)
            } else {
                unimplemented!("TODO Type Error in write-bytes")
            };
            stdout().write_all(bytes).unwrap();
            Value::Nil
        }
    }

    mod "math" as math {
        fn add[store, _k](l, r) {
            match (l, r) {
                (Value::Byte(l), Value::Byte(r)) => Value::Byte(l + r),
                (Value::Fixnum(l), Value::Fixnum(r)) => Value::Fixnum(l + r),
                _ => panic!("TODO Math Type Error {} + {}", l.display(store, false), r.display(store, false)),
            }
        }

        fn divide[store, _k](l, r) {
            match (l, r) {
                (Value::Byte(l), Value::Byte(r)) => Value::Byte(l / r),
                (Value::Fixnum(l), Value::Fixnum(r)) => Value::Fixnum(l / r),
                _ => panic!("TODO Math Type Error {} / {}", l.display(store, false), r.display(store, false)),
            }
        }

        fn modulo[store, _k](l, r) {
            match (l, r) {
                (Value::Byte(l), Value::Byte(r)) => Value::Byte(l % r),
                (Value::Fixnum(l), Value::Fixnum(r)) => Value::Fixnum(l % r),
                _ => panic!("TODO Math Type Error {} mod {}", l.display(store, false), r.display(store, false)),
            }
        }

        fn multiply[store, _k](l, r) {
            match (l, r) {
                (Value::Byte(l), Value::Byte(r)) => Value::Byte(l * r),
                (Value::Fixnum(l), Value::Fixnum(r)) => Value::Fixnum(l * r),
                _ => panic!("TODO Math Type Error {} * {}", l.display(store, false), r.display(store, false)),
            }
        }

        fn subtract[store, _k](l, r) {
            match (l, r) {
                (Value::Byte(l), Value::Byte(r)) => Value::Byte(l - r),
                (Value::Fixnum(l), Value::Fixnum(r)) => Value::Fixnum(l - r),
                _ => panic!("TODO Math Type Error {} - {}", l.display(store, false), r.display(store, false)),
            }
        }
    }

    mod "obj" as obj {
        fn get_type[_s, _k](val) {
            let s = match val {
                Value::Byte(_) => "byte",
                Value::Bytes(_,_) => "bytes",
                Value::Closure(_) | Value::Intrinsic(_) => "function",
                Value::Cons(_,_) => "cons",
                Value::Fixnum(_) => "fixnum",
                Value::Nil => "nil",
                Value::Object(ty, _) => ty.as_str(),
                Value::String(_, _) => "string",
                Value::Symbol(_) => "symbol",
                Value::Vector(_, _) => "vector",
            };
            Value::Symbol(s.into())
        }

        fn make_object[store, _k](ty, val) {
            typeck_name!(ty as Value::Symbol(ty));
            if !ty.contains(':') {
                panic!("Can't create object of reserved type {}", ty);
            }
            Value::Object(ty, store.store(val))
        }
    }

    mod "oftb" as oftb {
        fn read_dir[store, _k](path) {
            use std::fs::read_dir;

            let entries = {
                let path = if let Value::String(addr, len) = path {
                    store.get_str(addr, len)
                } else {
                    unimplemented!("TODO Type Error in read-dir")
                };

                let r = (|| {
                    read_dir(path)?
                        .map(|r| r.and_then(|e| {
                            e.file_type().map(|t| {
                                (e.path(), t)
                            })
                        }))
                        .collect::<Result<Vec<_>, _>>()
                })();
                match r {
                    Ok(entries) => entries,
                    Err(err) => panic!("(read_dir {:?}) -> {}", path, err),
                }
            };

            let entries = entries
                .into_iter()
                .map(|(p, t)| {
                    let t = if t.is_dir() {
                        "dir"
                    } else if t.is_symlink() {
                        "symlink"
                    } else {
                        "file"
                    };
                    Literal::Cons(
                        Box::new(Literal::String(p.display().to_string())),
                        Box::new(Literal::Symbol(t.into())))
                })
                .collect();
            store.store_literal(&Literal::list(entries))
        }

        fn read_file[store, _k](path) {
            let data = {
                let path = if let Value::String(addr, len) = path {
                    store.get_str(addr, len)
                } else {
                    unimplemented!("TODO Type Error in read-file")
                };
                match parse_file(path) {
                    Ok(data) => Literal::list(data),
                    Err(err) => panic!("(read_file {:?}) -> {}", path, err),
                }
            };
            store.store_literal(&data)
        }

        fn write_file[store, _k](path, data) {
            use std::fs::{create_dir_all, File};
            use std::io::Write;
            use std::path::Path;

            typeck_name!(path as Value::String(pa, pl), data as Value::Bytes(da, dl));
            let path: &Path = store.get_str(path.0, path.1).as_ref();
            let data = store.get_bytes(data.0, data.1);
            if let Some(dir) = path.parent() {
                create_dir_all(dir).ok();
            }
            let mut file = File::create(path).expect("Couldn't open file for writing");
            file.write_all(data).expect("Couldn't write to file");
            Value::Nil
        }
    }

    mod "strings" as strings {
        fn append[store, _k](l, r) {
            // TODO if l is the last string on the heap, just extend it
            let (la, ll, ra, rl) = match (l, r) {
                (Value::String(la, ll), Value::String(ra, rl)) => {
                    (la, ll, ra, rl)
                },
                _ => unimplemented!("TODO Type Error in string-append"),
            };

            let lan: usize = la.into();
            let ran: usize = ra.into();
            if lan + ll == ran {
                Value::String(la, ll + rl)
            } else {
                let s = {
                    let l = store.get_str(la, ll);
                    let r = store.get_str(ra, rl);
                    format!("{}{}", l, r)
                };
                let (addr, len) = store.store_str(&s);
                Value::String(addr, len)
            }
        }

        fn length[store, _k](s) {
            if let Value::String(a, l) = s {
                let s = store.get_str(a, l);
                let n = s.chars().count();
                Value::Fixnum(n as isize)
            } else {
                unimplemented!("TODO Type Error in string-length")
            }
        }

        fn slice[store, _k](start, end, s) {
            let (addr, len) = if let (
                Value::Fixnum(start_n),
                Value::Fixnum(end_n),
                Value::String(a, l),
            ) = (start, end, s)
            {
                let s = store.get_str(a, l);
                let start_n = start_n as usize;
                let end_n = end_n as usize;
                let start = s.char_indices().nth(start_n);
                let end = s.char_indices().nth(end_n);
                let a: usize = a.into();
                match (start, end) {
                    (Some((start, _)), Some((end, _))) => {
                        (a + start, end - start)
                    },
                    (Some((start, _)), None) if end_n == s.chars().count() => {
                        (a + start, end_n - start)
                    },
                    _ => unimplemented!(
                        "TODO out of bounds in string-slice (bounds of [{}, {}) on string {:?})",
                        start_n,
                        end_n,
                        s,
                    ),
                }
            } else {
                unimplemented!("TODO Type Error in string-slice")
            };
            Value::String(addr.into(), len)
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

    mod "vector" as vector {
        fn append[store, _k](l, r) {
            // TODO: Make this more optimized.
            match (l, r) {
                (Value::Vector(la, ll), Value::Vector(ra, rl)) => {
                    let mut vals = store.get_vec(la, ll).into_iter()
                        .map(|val| store.store(val))
                        .collect::<Vec<_>>();
                    vals.extend(store.get_vec(ra, rl).into_iter()
                        .map(|val| store.store(val)));
                    let (a, l) = store.store_vec(&vals);
                    Value::Vector(a, l)
                }
                _ => unimplemented!("TODO type error in vector-append")
            }
        }

        fn length[_s, _k](s) {
            if let Value::Vector(_, l) = s {
                Value::Fixnum(l as isize)
            } else {
                unimplemented!("TODO Type Error in vector-length")
            }
        }

        fn make[store, konts](func, len) {
            typeck_name!(len as Value::Fixnum(len));
            if len <= 0 {
                let (addr, len) = store.store_vec(&[]);
                Value::Vector(addr, len)
            } else {
                let mut konts = konts;
                konts.push(Kont::MakeVector(0, (len - 1) as usize, func, Vec::new()));
                return ::interpreter::eval::apply(func, vec![Value::Fixnum(0)], store, konts);
            }
        }

        fn nth[store, _k](n, s) {
            if let (Value::Fixnum(n), Value::Vector(a, l)) = (n, s) {
                let n = n as usize;
                if n < l {
                    let a: usize = a.into();
                    store.get_vec_val(Addr::from(a + n))
                } else {
                    unimplemented!("TODO out of bounds in vector-nth")
                }
            } else {
                unimplemented!("TODO Type Error in vector-nth")
            }
        }

        fn slice[_s, _k](start, end, s) {
            let (a, l) = if let (
                Value::Fixnum(start),
                Value::Fixnum(end),
                Value::Vector(a, l),
            ) = (start, end, s)
            {
                let start = start as usize;
                let end = end as usize;
                if start <= l && end <= l && start <= end {
                    let a: usize = a.into();
                    (a + start as usize, end - start)
                } else {
                    unimplemented!(
                        "TODO out of bounds in vector-slice (bounds of [{}, {}) on vector of length {})",
                        start,
                        end,
                        l,
                    )
                }
            } else {
                unimplemented!("TODO Type Error in vector-slice")
            };

            Value::Vector(a.into(), l)
        }
    }
}
