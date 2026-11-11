#![doc = include_str!("../README.md")]

mod host;
mod identity;
mod scope;
mod term;

pub use self::{
    host::Host,
    identity::{Hierarchy, Marker, MarkerRef, NodeId},
    scope::{HostScope, Scope, ScopeCx},
    term::{ErasedTerm, Term, TermIter},
};

/// Call `$m!(A, B, ...)` for each non-empty prefix of the identifier list.
///
/// # Syntax
///
/// ```
/// # use expie_elab::for_each_tuple;
/// # macro_rules! macro_name { ($($args:ident),*) => {;} }
/// for_each_tuple!(macro_name for (A, B, C));
/// ```
///
/// This calls:
///
/// ```
/// # macro_rules! macro_name { ($($args:ident),*) => {;} }
/// macro_name!(A);
/// macro_name!(A, B);
/// macro_name!(A, B, C);
/// ```
///
/// Or optionally:
///
/// ```
/// # use expie_elab::for_each_tuple;
/// # macro_rules! macro_name { ($($args:ident),* @ Type) => {;} }
/// for_each_tuple!(macro_name for (A, B, C) @ Type);
/// ```
///
/// ...which appends `@ Type` to the end of the input to the macro, as in:
///
/// ```
/// # macro_rules! macro_name { ($($args:ident),* @ Type) => {;} }
/// macro_name!(A @ Type);
/// macro_name!(A, B @ Type);
/// macro_name!(A, B, C @ Type);
/// ```
///
/// `Type` can be any identifier (but not any type) – this allows for more
/// expressive token manipulation than with `$_:ty`.
///
/// It is recommended to name the target macro by prefixing it with `impl_*` –
/// this makes macro usage read nicer.
#[macro_export]
macro_rules! for_each_tuple {
    // Expand the 1-tuple, then recurse (with the typed payload if provided).
    ($m:ident for ($head:ident $(, $tail:ident)* $(,)?) $(@ $ty:ident)?) => {
        $m!($head $(@ $ty)?);
        $crate::for_each_tuple!(@inner $m; ($head); ($($tail),*) $(; $ty)?);
    };
    // Grow the typed prefix by one id, expand, recurse.
    (@inner $m:ident; ($($prefix:ident),+); ($next:ident $(, $tail:ident)*) $(; $ty:ident)?) => {
        $m!($($prefix),+, $next $(@ $ty)?);
        $crate::for_each_tuple!(@inner $m; ($($prefix),+, $next); ($($tail),*) $(; $ty)?);
    };
    // No more tail.
    (@inner $m:ident; ($($prefix:ident),+); () $(; $ty:ident)?) => {};
}

/// Call [`for_each_tuple`] with the provided macro for tuples of arity <=12.
///
/// Tuples use types up to: `(A, B, C, D, E, F, G, H, I, J, K, L)`.
///
/// This version also accepts `@ Type` syntax.
#[macro_export]
macro_rules! for_each_tuple_12 {
    ($m:ident $(@ $ty:ident)?) => {
        $crate::for_each_tuple!($m for (A, B, C, D, E, F, G, H, I, J, K, L) $(@ $ty)?);
    };
}
