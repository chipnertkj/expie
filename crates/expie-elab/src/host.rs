//! Lowering target provider for [`Term::lower`].
//!
//! [`Term::lower`]: crate::Term::lower

use crate::Hierarchy;

/// A host is the target provider for [`Term`] lowering.
///
/// Its responsibility is to provide arbitrary capabilities, which allow complex
/// terms to [`lower`] domain-specific details into the host's [`Store`].
///
/// A term may require a generic host `H` to provide capabilities for lowering
/// the term, by restricting an implementation with [`H: HasLayer<L>`], where
/// `L` is a concrete [`Layer`] type.
///
/// [`Term`]: crate::Term
/// [`Store`]: Self::Store
/// [`lower`]: crate::Term::lower
/// [`H: HasLayer<L>`]: crate::HasLayer
/// [`Layer`]: crate::Layer
pub trait Host {
    /// This is the type returned by [`new_store`].
    ///
    /// It is the target for all mutating operations performed by [`Term`]s
    /// during [`Term::lower`].
    ///
    /// A `Store` is the collection of unprocessed lowered data provided by
    /// terms.
    ///
    /// [`new_store`]: Self::new_store
    /// [`Term`]: crate::Term
    /// [`Term::lower`]: crate::Term::lower
    type Store<'id>;
    /// This is the type returned by [`finalize_scope`].
    ///
    /// It is the final output of the host, after all [`Term`]s have been
    /// lowered through a [`Scope`], and after the final [`Store`] value
    /// (acquired by destructuring `Scope`) has been consumed by
    /// `finalize_scope`, in order to refine the result.
    ///
    /// This type may be set to *be* [`Self::Store`], if no further refinement
    /// step is desired after all terms have been lowered.
    ///
    /// [`Scope`]: crate::Scope
    /// [`finalize_scope`]: Self::finalize_scope
    /// [`Store`]: Self::Store
    /// [`Term`]: crate::Term
    /// [`Term::lower`]: crate::Term::lower
    type Output<'id>;

    /// Create a new [`Store`] for a new [`Scope`].
    ///
    /// A new store is allocated for each
    /// [`scope`](crate::scope::HostScope::scope) call. Scopes are the
    /// instrument through which terms mutate the host store.
    ///
    /// [`Scope`]: crate::Scope
    /// [`Store`]: Self::Store
    fn new_store<'id>(&self) -> Self::Store<'id>;

    /// Consume a deconstructed [`Scope`] and return the final [`Output`] of the
    /// host.
    ///
    /// The scope owned the store created for it. After all [`Term`]s have
    /// called [`Term::lower`] on that scope, the host is allowed to refine
    /// store state into `Output`, if desired (if types differ).
    ///
    /// [`Output`]: Self::Output
    /// [`Scope`]: crate::Scope
    /// [`Store`]: Self::Store
    /// [`Term`]: crate::Term
    /// [`Term::lower`]: crate::Term::lower
    fn finalize_scope<'id>(
        &self,
        hierarchy: Hierarchy<'id>,
        store: Self::Store<'id>,
    ) -> Self::Output<'id>
    where
        Self: Sized;
}
