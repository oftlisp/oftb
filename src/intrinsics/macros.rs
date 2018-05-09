/// Counts the token trees provided as input.
macro_rules! __intrinsics_count {
    () => { 0 };
    ($h:tt $($t:tt)*) => {
        1 + __intrinsics_count!($($t)*)
    };
}

/// Given an argument variable and an arguments specification, ensures that the
/// arguments match the specification, returning the number of fixed arguments.
macro_rules! __intrinsics_argn {
    ($args_var:ident, $($args:ident)*) => {
        // TODO: Return a real error.
        assert_eq!($args_var.len(), __intrinsics_count!($($args)*));
    };
    ($args_var:ident, $($args:ident)* ... $rest:ident) => {
        // TODO: Return a real error.
        assert!($args_var.len() >= __intrinsics_count!($($args)*));
    };
}

macro_rules! __intrinsics_args_1 {
    ($args_var:ident, $n:expr,) => {};
    ($args_var:ident, $n:expr, $h:ident $($t:tt)*) => {
        let $h = &$args_var[$n];
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
    ($args_var:ident, $($args:ident)*) => {
        __intrinsics_args_2!($args_var, $($args)*)
    };
    ($args_var:ident, $($args:ident)* ... $rest:ident) => {
        __intrinsics_args_2!($args_var, $($args)*, $rest)
    };
}

macro_rules! __intrinsics_name {
    ($name:ident) => { stringify!($name) };
    ($name:expr) => { $name };
}

macro_rules! intrinsics {
    (
        mod $mod_name:tt;
        $(fn $name:ident[$store:ident, $konts:ident]($($args:tt)*) $body:block)*
    ) => {
        $(fn $name<'program>(
            args: $crate::std::vec::Vec<$crate::interpreter::Value>,
            $store: &$crate::interpreter::Store<'program>,
            $konts: $crate::std::vec::Vec<$crate::interpreter::Kont<'program>>,
        ) -> $crate::interpreter::State<'program> {
            __intrinsics_argn!(args, $($args)*);
            __intrinsics_args!(args, $($args)*);
            let value = $body;
            $crate::interpreter::eval::kontinue(value, $konts)
        })*

        lazy_static! {
            /// The values of the intrinsics.
            static ref INTRINSICS: $crate::std::collections::HashMap<
                $crate::symbol::Symbol,
                for<'program> fn(
                    $crate::std::vec::Vec<$crate::interpreter::Value>,
                    &$crate::interpreter::Store<'program>,
                    $crate::std::vec::Vec<$crate::interpreter::Kont<'program>>,
                ) -> $crate::interpreter::State<'program>
            > = {
                let mut hm = $crate::std::collections::HashMap::with_capacity(
                    __intrinsics_count!($($name)*)
                );
                $(hm.insert(
                    $crate::symbol::Symbol::from(stringify!($name)),
                    $name as for<'program> fn(
                        $crate::std::vec::Vec<$crate::interpreter::Value>,
                        &$crate::interpreter::Store<'program>,
                        $crate::std::vec::Vec<$crate::interpreter::Kont<'program>>,
                    ) -> $crate::interpreter::State<'program>,
                );)*
                hm
            };

            /// The intrinsics declared.
            static ref INTRINSIC_DECLS: $crate::std::collections::HashSet<
                $crate::symbol::Symbol,
            > = {
                let mut hs = $crate::std::collections::HashSet::with_capacity(
                    __intrinsics_count!($($name)*)
                );
                $(hs.insert($crate::symbol::Symbol::from(stringify!($name)));)*
                hs
            };

            /// The name of the declared module.
            static ref MOD_NAME: $crate::symbol::Symbol =
                $crate::symbol::Symbol::from(__intrinsics_name!($mod_name));
        }
    };
}
