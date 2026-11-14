//! [`Scope`] pass effect system.
//!
//! [`Scope`]: crate::Scope
//!
//! An *effect* is a singular data point – it encodes additional information
//! about the data [lowered] to a [`Store`].
//!
//! [lowered]: crate::Term::lower
//! [`Store`]: crate::Host::Store
//!
//! All effect must be *bound* to the scope through the means of an
//! [`EffectDialect`], which defines the binding process via
//! [`Binding`].
//!
//! [`Binding`]: EffectDialect::Binding
//!
//! The effect system provides capabilities for constraining parts of a generic
//! [`Term`] structure to a particular effect type `E` and dialect type `D`, as
//! well as ways to map between different effect types for the purposes of
//! enabling *effect encapsulation* via [`map_effect`].
//!
//! [`map_effect`]: EffectDialect::map_effect

use std::convert::Infallible;

use crate::{Host, ScopeCx, Term};

/// An *effect dialect* defines how [effects](self) are bound to the
/// [`Scope`].
///
/// [`Scope`]: crate::Scope
///
/// The [`ScopeCx`] is able to modify the [`Effects`] structure for an open
/// scope. It is generic over both the dialect `D` and the effect type `E`,
/// but only the effect type may change over the course of a scope – the dialect
/// `D` is bound by the [`scope`](crate::scope::HostScope::scope) call.
///
/// The dialect itself defines an associated [`Binding`] type, which is used to
/// encode an effect – this is referred to as *effect binding*.
///
/// [`Binding`]: EffectDialect::Binding
///
/// Bindings are meant to be used as wrappers for the actual effect type `E`
/// (bindings are generic over `E`). Once `Effects` reaches
/// [`Host::finalize_scope`] (after all [`Term`]s have been lowered), `Binding`
/// informs the host on how to interpret the effects that have been lowered into
/// `Effects` under this dialect.
///
/// A dialect also defines the protocol for mapping between bindings of
/// different effect types `E`. This is done by [`Self::map_effect`].
pub trait EffectDialect {
    /// An *effect binding* encodes an effect `E` under this dialect.
    ///
    /// The binding type informs the host on how to interpret an effect provided
    /// to [`Effects`] by the [`ScopeCx`].
    type Binding<E>: std::fmt::Debug;

    /// Provides a mapping operation between [`Binding`]s of different effect
    /// types `E`. Here, `Old` is the effect type of the original
    /// binding, and `New` is the effect type of the mapped binding.
    ///
    /// [`Binding`]: Self::Binding
    fn map_effect<Old, New, F>(binding: Self::Binding<Old>, map: F) -> Self::Binding<New>
    where
        F: Clone + Fn(Old) -> New;
}

/// Infallible [`EffectDialect`] implementation for [`Infallible`].
///
/// Since [`Binding`] is also infallible, no binding can ever be constructed
/// under this dialect – effect type `E` is trivially redundant, while mapping
/// operations are unreachable.
///
/// [`Binding`]: EffectDialect::Binding
impl EffectDialect for Infallible {
    type Binding<E> = Self;

    fn map_effect<Old, New, F>(binding: Self::Binding<Old>, _map: F) -> Self::Binding<New>
    where
        F: Fn(Old) -> New,
    {
        match binding {}
    }
}

/// A collection of [effects](self) *bound* to the [`Scope`] for a particular
/// effect type `E` and [`EffectDialect`] `D`.
///
/// [`Scope`]: crate::Scope
///
/// The current effect type `E` can be mapped (according to the dialect) via
/// [`Self::map_effect`]. The dialect `D` remains the same across the entire
/// [`scope`](crate::scope::HostScope::scope) call.
///
/// These effects are forwarded to the [`Host`], specifically
/// [`Host::finalize_scope`], once all [`Term`]s have been lowered for a given
/// scope. The host is then responsible for interpreting the effects, in
/// accordance with the rules specified by the [`EffectDialect::Binding`].
#[derive(Debug)]
pub struct Effects<D: EffectDialect, E> {
    /// All effects pushed to this collection.
    entries: Vec<D::Binding<E>>,
}

impl<D: EffectDialect, E> Effects<D, E> {
    /// Create an empty collection of effects.
    #[must_use = "constructor does not have side effects"]
    pub(crate) const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Map the current effect type `E` to a new effect type `New` according to
    /// the [`EffectDialect`].
    pub(crate) fn map_effect<New, F>(self, map: F) -> Effects<D, New>
    where
        F: Clone + Fn(E) -> New,
    {
        Effects {
            entries: self
                .entries
                .into_iter()
                .map(|binding| D::map_effect(binding, map.clone()))
                .collect(),
        }
    }

    /// Push a new [`Binding`] to the collection.
    ///
    /// [`Binding`]: EffectDialect::Binding
    pub(crate) fn push(&mut self, binding: D::Binding<E>) {
        self.entries.push(binding);
    }

    /// Extend the collection with the contents of an iterator.
    pub(crate) fn extend(&mut self, bindings: impl IntoIterator<Item = D::Binding<E>>) {
        self.entries.extend(bindings);
    }

    /// Get an iterator over the collection of effects.
    pub fn entries(&self) -> impl Iterator<Item = &D::Binding<E>> {
        self.entries.iter()
    }

    /// Get the number of effects in the collection.
    #[must_use = "result of a length check"]
    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the collection is empty (has no effects).
    #[must_use = "result of a length check"]
    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl<D: EffectDialect, E> IntoIterator for Effects<D, E> {
    type Item = D::Binding<E>;
    type IntoIter = std::vec::IntoIter<D::Binding<E>>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
    }
}

/// A [`Term`] that wraps the inner term, providing a mapping operation between
/// [`Binding`]s of the inner effect type `Old` and the outer effect type `New`.
///
/// [`Binding`]: EffectDialect::Binding
#[derive(Debug)]
pub struct MapEffect<T, F, Old> {
    /// The inner term.
    inner: T,
    /// The mapping function applied to the inner effect type `Old`.
    ///
    /// This must output an outer effect type `New` for the [`Term`]
    /// implementation to hold (though `New` itself is not constrained).
    map: F,
    /// `Old` must use a phantom type marker.
    /// This type parameter is provided to the [`Term`] implementation to ensure
    /// that the mapping function `F` can be generic over the inner effect type
    /// `Old`.
    _phantom: std::marker::PhantomData<fn(Old) -> Old>,
}

impl<'id, H, D, New, Old, T, F> Term<'id, H, D, New> for MapEffect<T, F, Old>
where
    F: Clone + Fn(Old) -> New,
    T: Term<'id, H, D, Old>,
{
    fn lower(self, cx: &mut ScopeCx<'id, '_, H, D, New>)
    where
        H: Host,
        D: EffectDialect,
    {
        let Self {
            inner,
            map,
            _phantom: _,
        } = self;
        cx.map_effect(map, |inner_cx| {
            inner.lower(inner_cx);
        });
    }
}

/// An extension trait for [`Term`]s that allows for wrapping the term with
/// [`MapEffect`].
pub trait TermEffectMapping<'id, H, D, Old> {
    /// Wrap the term `self` with [`MapEffect`], applying a mapping function
    /// `map` from the inner effect type `Old` to the outer effect type `New`.
    fn map_effect<New, F>(self, map: F) -> MapEffect<Self, F, Old>
    where
        Self: Sized,
        H: Host,
        D: EffectDialect;
}

impl<'id, H, D, Inner, T> TermEffectMapping<'id, H, D, Inner> for T
where
    Self: Term<'id, H, D, Inner>,
{
    fn map_effect<Outer, F>(self, map: F) -> MapEffect<Self, F, Inner>
    where
        Self: Sized,
        H: Host,
        D: EffectDialect,
    {
        MapEffect {
            inner: self,
            map,
            _phantom: std::marker::PhantomData,
        }
    }
}
