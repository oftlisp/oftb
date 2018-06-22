/// Counts the token trees provided as input.
macro_rules! __intrinsics_count {
    () => { 0 };
    ($h:tt $($t:tt)*) => {
        1 + __intrinsics_count!($($t)*)
    };
}

/// Given an argument variable and an arguments specification, ensures that the
/// arguments match the specification, returning the number of fixed arguments.
///
/// TODO: Return a real error.
macro_rules! __intrinsics_argn {
    // These two cases are just to silence a warning.
    ($name:ident, $args_var:ident,) => {};
    ($name:ident, $args_var:ident, *$rest:ident) => {};

    ($name:ident, $args_var:ident, $($args:ident),*) => {
        assert_eq!($args_var.len(), __intrinsics_count!($($args)*), concat!("bad argn to ", stringify!($name)));
    };
    ($name:ident, $args_var:ident, $($args:ident,)* *$rest:ident) => {
        assert!($args_var.len() >= __intrinsics_count!($($args)*), concat!("bad argn to ", stringify!($name)));
    };
}

macro_rules! __intrinsics_args_1 {
    ($args_var:ident, $n:expr,) => {};
    ($args_var:ident, $n:expr, $h:ident $($t:tt)*) => {
        let $h = $args_var[$n];
        __intrinsics_args_1!($args_var, $n + 1, $($t)*);
    };
}

macro_rules! __intrinsics_args_2 {
    ($args_var:ident, $($args:ident)*) => {
        __intrinsics_args_1!($args_var, 0, $($args)*);
    };
    ($args_var:ident, $($args:ident)*, $rest:ident) => {
        __intrinsics_args_2!($args_var, $($args)*);
        let $rest = &$args_var[__intrinsics_count!($($args)*)..];
    };
}

/// Creates bindings from the given argument names to arguments in the given
/// argument list.
macro_rules! __intrinsics_args {
    ($args_var:ident, $($args:ident),*) => {
        __intrinsics_args_2!($args_var, $($args)*)
    };
    ($args_var:ident, $($args:ident,)* *$rest:ident) => {
        __intrinsics_args_2!($args_var, $($args)*, $rest)
    };
}

macro_rules! __intrinsics_mod_name {
    ($pkg:expr,"") => {
        $pkg
    };
    ($pkg:expr, $mod:expr) => {
        concat!($pkg, "/", $mod)
    };
}

macro_rules! __intrinsics_mod {
    (
        mod_name $mod_name:expr;
        $(fn $name:ident[$store:ident, $konts:ident]($($args:tt)*) $body:block)*
    ) => {
        #[allow(unused_imports)]
        use super::*;

        $(fn $name<'program>(
            _args: $crate::std::vec::Vec<$crate::interpreter::Value>,
            $store: &mut $crate::interpreter::Store<'program>,
            $konts: $crate::std::vec::Vec<$crate::interpreter::Kont<'program>>,
        ) -> $crate::interpreter::State<'program> {
            #[allow(unused_macros)]
            macro_rules! __intrinsics_name { () => (stringify!($name)) }
            __intrinsics_argn!($name, _args, $($args)*);
            __intrinsics_args!(_args, $($args)*);
            #[allow(unreachable_code)]
            $crate::interpreter::eval::kontinue($body, $konts)
        })*

        lazy_static! {
            pub static ref DECLARED: $crate::std::collections::HashSet<
                $crate::symbol::Symbol,
            > = {
                let mut hs = $crate::std::collections::HashSet::with_capacity(
                    __intrinsics_count!($($name)*)
                );
                $(hs.insert($crate::symbol::Symbol::from(stringify!($name)));)*
                hs
            };

            pub static ref VALUES: $crate::std::collections::HashMap<
                $crate::symbol::Symbol,
                $crate::interpreter::Value,
            > = {
                let mut hm = $crate::std::collections::HashMap::with_capacity(
                    __intrinsics_count!($($name)*)
                );
                $(hm.insert(
                    $crate::symbol::Symbol::from(concat!(
                        $mod_name,
                        ":",
                        stringify!($name),
                    )),
                    $crate::interpreter::Value::Intrinsic(
                        $crate::interpreter::Intrinsic($name),
                    ),
                );)*
                hm
            };
        }
    };
}

macro_rules! intrinsics {
    (
        pkg $pkg_name:tt as $ty_name:ident;
        $(mod $module_name:tt as $mod_name:ident { $($mod_vals:tt)* })*
    ) => {
        pub enum $ty_name {}

        $(mod $mod_name {
            __intrinsics_mod! {
                mod_name __intrinsics_mod_name!($pkg_name, $module_name);
                $($mod_vals)*
            }
        })*

        impl $crate::BuiltinPackage for $ty_name {
            fn decls() -> $crate::std::collections::HashMap<
                $crate::symbol::Symbol,
                $crate::std::collections::HashSet<$crate::symbol::Symbol>,
            > {
                let mut hm = $crate::std::collections::HashMap::new();
                $(hm.insert(
                    $crate::symbol::Symbol::from($module_name),
                    $mod_name::DECLARED.clone(),
                );)*
                hm
            }

            fn name() -> $crate::symbol::Symbol {
                $crate::symbol::Symbol::from($pkg_name)
            }

            fn values() -> $crate::std::collections::HashMap<
                $crate::symbol::Symbol,
                $crate::interpreter::Value,
            > {
                let mut hm = $crate::std::collections::HashMap::new();
                $(hm.extend($mod_name::VALUES.clone());)*
                hm
            }
        }
    }
}

macro_rules! typeck_name {
    ($($id:ident as Value :: $variant:ident $args:tt),*) => {
        $(let $id = if let Value::$variant $args = $id {
            $args
        } else {
            unimplemented!("TODO typeck_var failed in {}", __intrinsics_name!())
        };)*
    };
}
