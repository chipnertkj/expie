//! Abstract Syntax Tree (AST) term definition.
//!
//! # Responsibility
//!
//! A term acts as the translation instrument for the *sources of information*
//! (mentioned in [`Term::lower`]'s documentation) into data specific to a
//! [`Host::Store`].
//!
//! Because of the definition of `lower`, *lowering* can be thought of
//! as a form of *elaboration* to the scope, on what a term's intent is. Both
//! "terms" (in the sense of *terminology*, i.e. elaboration and
//! lowering) refer to the same action, but express very different
//! ideas.
//!
//! Elaboration is the process of building up an understanding of the
//! desired result, by filling in details to achieve an outcome. Lowering,
//! on the other hand, strips away semantic details and translates them into
//! instructions for the next step in the overall process.

use std::convert::Infallible;

use crate::{Host, ScopeCx, effect::EffectDialect, for_each_tuple_12};

/// A term is a unit of data in an Abstract Syntax Tree (AST).
///
/// `expie-base` does not specify the actual AST in use.
/// This architecture instead allows us to represent arbitrary declarative trees
/// as compile-time syntax, which can be lowered down into runtime details.
///
/// Declaring a term only requires that it knows how to [`lower`] itself down
/// into the appropriate implementation details specific to a particular
/// [`Host`].
///
/// [`lower`]: Self::lower
///
/// A term is supposed to work as an abstraction of a domain-specific feature.
/// Its fields / variants and methods should act as an interface in
/// communicating intent. `lower` provides the glue between that ideal and the
/// reality of the software components a term integrates with.
///
/// Terms are parameterized by the [`EffectDialect`] `D` and the effect type
/// `E`. Please see the [`effect`] module for more information on the effects
/// system.
///
/// [`effect`]: crate::effect
pub trait Term<'id, H, D, E> {
    /// Lower this term inside a [`ScopeCx`].
    ///
    /// *Lowering* is the process of mutating the [`Scope`] (which the scope
    /// context borrows) by providing it with additional information.
    ///
    /// [`Scope`]: crate::Scope
    ///
    /// A term writes information to the [`Host::Store`] (owned by the scope)
    /// via methods exposed on the scope context.
    ///
    /// Conceptually, this information should come from:
    ///   1. The occurrence of this term in the lowering process.
    ///   2. The data contained within this term's value.
    ///
    /// # Identity and hierarchy
    ///
    /// In order for lowered data to maintain identity (i.e. refer back to the
    /// term it originates from), the scope provides identity registration
    /// via [`ScopeCx::register`]. This allocates a unique [`NodeId`] and opens
    /// a new [`ScopeCx`], with the new `Node` set as the *current target*.
    ///
    /// [`NodeId`]: crate::NodeId
    ///
    /// The newly constructed context is available only within a closure, which
    /// is passed to `register` by the caller.
    ///
    /// This pattern allows for constructing hierarchies of nodes, as any node
    /// registered using that inner context will be treated as a child of the
    /// current target node. This parent-child tree hierarchy may be later
    /// queried by the [`Host`] to resolve relationships.
    ///
    /// [`Host`]: crate::Host
    ///
    /// # Term kinds
    ///
    /// Note that registration is completely optional. A term may contribute
    /// information without registering as an individual entity –
    /// you may refer to this pattern as a **"virtual term"**.
    ///
    /// A term that doesn't register itself, but lowers multiple terms under the
    /// same target node is analogously a **"sequence term"** – it expands to a
    /// series of terms that may register their own identities. They get
    /// lowered in the order defined by the sequence term. An
    /// example of this is the term implementation for tuples.
    ///
    /// It is possible to implement a term that registers multiple identities
    /// for itself. This is also valid – it declares multiple nodes, each a
    /// target for different child edges. In such cases, it is up to the
    /// host and that term to correctly orchestrate and interpret those
    /// relationships. This pattern may be referred to as a **"polysemous
    /// term"**.
    fn lower(self, cx: &mut ScopeCx<'id, '_, H, D, E>)
    where
        H: Host,
        D: EffectDialect;
}

/// Dyn-compatible lowering of [`Term`]s via [`Box`].
///
/// [`Term::lower`] takes `self` as a parameter, which implies `Self: Sized`.
/// This prevents types like `dyn Term` from being used in regular term
/// position.
///
/// `ErasedTerm` provides an escape hatch for this limitation, allowing one to
/// take a boxed `Self` term and lower it.
///
/// Instead of taking a `Box<dyn Term>` as an argument, `Term` in
/// [`ErasedTerm::lower_boxed`] is replaced with a boxed `ErasedTerm` / `Self`,
/// enabling this pattern. Compatibility with `Term` is achieved via blanket
/// implementation of `ErasedTerm` for every [`Sized`] term.
pub trait ErasedTerm<'id, H, D, E> {
    /// [`Term::lower`] for a [`?Sized`](Sized) [`Term`] *equivalent* –
    /// it takes a boxed [`ErasedTerm`] as the argument instead.
    fn lower_boxed(self: Box<Self>, cx: &mut ScopeCx<'id, '_, H, D, E>)
    where
        H: Host,
        D: EffectDialect;
}

/// Blanket implementation of [`ErasedTerm`] for every [`Sized`] [`Term`].
impl<'id, H, D, E, T> ErasedTerm<'id, H, D, E> for T
where
    Self: Term<'id, H, D, E>,
{
    fn lower_boxed(self: Box<Self>, cx: &mut ScopeCx<'id, '_, H, D, E>)
    where
        H: Host,
        D: EffectDialect,
    {
        // Because T is Sized here, (*self) moves T out of the box cleanly.
        (*self).lower(cx);
    }
}

/// Blanket implementation of [`Term`] for every [`ErasedTerm`] [`Box`].
impl<'id, H, D, E, T> Term<'id, H, D, E> for Box<T>
where
    T: ErasedTerm<'id, H, D, E>,
{
    fn lower(self, cx: &mut ScopeCx<'id, '_, H, D, E>)
    where
        H: Host,
        D: EffectDialect,
    {
        self.lower_boxed(cx);
    }
}

/// Type alias for a trait object of [`ErasedTerm`] with an additional trait
/// object lifetime `'id`.
pub type DynTerm<'id, H, D, E> = dyn ErasedTerm<'id, H, D, E> + 'id;

/// Type alias for a [`DynTerm`] that produces no [effects] (`E = Infallible`)
/// and does not support any [`EffectDialect`] (`D = Infallible`).
///
/// [effects]: crate::effect
pub type PureDynTerm<'id, H> = DynTerm<'id, H, Infallible, Infallible>;

/// Extension trait providing type erasure for [`Term`]s.
pub trait TermErasure {
    /// Type erase the [`Term`] `self` into a boxed [`ErasedTerm`].
    fn erase<'id, H, D, E>(self) -> Box<DynTerm<'id, H, D, E>>
    where
        Self: Term<'id, H, D, E> + 'id,
        H: Host,
        D: EffectDialect;
}

impl<T> TermErasure for T {
    fn erase<'id, H, D, E>(self) -> Box<DynTerm<'id, H, D, E>>
    where
        Self: Term<'id, H, D, E> + 'id,
        H: Host,
        D: EffectDialect,
    {
        Box::new(self)
    }
}

/// Wrapper newtype.
/// Allows usage of any [`IntoIterator`] – where [`Item`] is a [`Term`] – as a
/// sequence term.
///
/// Implementing term for `I: IntoIterator` directly is not possible due to
/// [trait coherence rules].
///
/// [`Item`]: IntoIterator::Item
/// [trait coherence rules]: https://doc.rust-lang.org/reference/items/implementations.html#r-items.impl.trait.orphan-rule.def
#[derive(Debug, derive_more::From)]
pub struct TermIter<I>(
    /// Inner iterable.
    pub I,
);

/// Implement sequence [`Term`] for [`TermIter`].
impl<'id, H, D, E, T, I> Term<'id, H, D, E> for TermIter<I>
where
    T: Term<'id, H, D, E>,
    I: IntoIterator<Item = T>,
{
    fn lower(self, cx: &mut ScopeCx<'id, '_, H, D, E>)
    where
        H: Host,
        D: EffectDialect,
    {
        for item in self.0 {
            item.lower(cx);
        }
    }
}

/// Enable `.into_iter()` / `.collect()` directly on [`TermIter`].
impl<A, I> FromIterator<A> for TermIter<I>
where
    I: FromIterator<A>,
{
    fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self {
        Self(I::from_iter(iter))
    }
}

/// Implement [`IntoIterator`] for [`TermIter`].
impl<I, T> IntoIterator for TermIter<I>
where
    I: IntoIterator<Item = T>,
{
    type Item = T;
    type IntoIter = I::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// Implement sequence [`Term`] for a tuple of terms.
macro_rules! impl_tuple_term {
    ($($name:ident),* $(,)?) => {
        impl<'id, HOST, DIALECT, EFFECT, $($name),*> Term<'id, HOST, DIALECT, EFFECT> for ($($name),*,)
        where
            $($name: Term<'id, HOST, DIALECT, EFFECT>),*
        {
            fn lower(self, cx: &mut ScopeCx<'id, '_, HOST, DIALECT, EFFECT>)
            where
                HOST: Host,
                DIALECT: EffectDialect,
            {
                #[allow(non_snake_case, reason = "macro expansion")]
                let ($($name),*,) = self;
                $($name.lower(cx);)*
            }
        }
    };
}

for_each_tuple_12!(impl_tuple_term);

/// Implement sequence [`Term`] for [`Option`] of a term.
impl<'id, H, D, E, T> Term<'id, H, D, E> for Option<T>
where
    T: Term<'id, H, D, E>,
{
    fn lower(self, cx: &mut ScopeCx<'id, '_, H, D, E>)
    where
        H: Host,
        D: EffectDialect,
    {
        if let Some(term) = self {
            term.lower(cx);
        }
    }
}

/// Implement sequence [`Term`] for an array of terms.
impl<'id, H, D, E, T, const N: usize> Term<'id, H, D, E> for [T; N]
where
    T: Term<'id, H, D, E>,
{
    fn lower(self, cx: &mut ScopeCx<'id, '_, H, D, E>)
    where
        H: Host,
        D: EffectDialect,
    {
        for term in self {
            term.lower(cx);
        }
    }
}

/// Implement sequence [`Term`] for a [`Vec`] of terms.
impl<'id, H, D, E, T> Term<'id, H, D, E> for Vec<T>
where
    T: Term<'id, H, D, E>,
{
    fn lower(self, cx: &mut ScopeCx<'id, '_, H, D, E>)
    where
        H: Host,
        D: EffectDialect,
    {
        for term in self {
            term.lower(cx);
        }
    }
}
