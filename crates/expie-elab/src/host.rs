//! Lowering target provider for [`Term::lower`].
//!
//! [`Term::lower`]: crate::Term::lower

use crate::{EffectDialect, Effects, Hierarchy};

/// A host is the target provider for [`Term`] lowering.
///
/// Its responsibility is to provide arbitrary capabilities, which allow complex
/// terms to [`lower`] domain-specific details into the host's [`Store`].
///
/// [`Term`]: crate::Term
/// [`Store`]: Self::Store
/// [`lower`]: crate::Term::lower
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
    /// This is the type returned by [`finalize_pass`].
    ///
    /// It is the final output of the host, after all [`Term`]s have been
    /// lowered through a [`Pass`], and after the final [`Store`] value
    /// (acquired by destructuring `Pass`) has been consumed by
    /// `finalize_pass`, in order to refine the result.
    ///
    /// This type may be set to *be* [`Self::Store`], if no further refinement
    /// step is desired after all terms have been lowered.
    ///
    /// [`Pass`]: crate::Pass
    /// [`finalize_pass`]: Self::finalize_pass
    /// [`Store`]: Self::Store
    /// [`Term`]: crate::Term
    /// [`Term::lower`]: crate::Term::lower
    type Output<'id, D, E>
    where
        D: EffectDialect + 'id,
        E: 'id;

    /// Create a new [`Store`] for a new [`Pass`].
    ///
    /// A new store is allocated for each pass. Passes are the
    /// instrument through which terms mutate the host store.
    ///
    /// [`Pass`]: crate::Pass
    /// [`Store`]: Self::Store
    fn new_store<'id>(&self) -> Self::Store<'id>;

    /// Consume a deconstructed [`Pass`] and return the final [`Output`] of the
    /// host.
    ///
    /// The pass owned the store created for it. After all [`Term`]s have
    /// called [`Term::lower`] on that pass, the host is allowed to refine
    /// store state into `Output`, if desired (if types differ – otherwise just
    /// return the store here).
    ///
    /// The host may also act upon any [`Effects`] pushed to the pass.
    /// This is usually used for executing side effects, on top of the resulting
    /// store.
    ///
    /// [`Output`]: Self::Output
    /// [`Pass`]: crate::Pass
    /// [`Store`]: Self::Store
    /// [`Term`]: crate::Term
    /// [`Term::lower`]: crate::Term::lower
    fn finalize_pass<'id, D, E>(
        &self,
        hierarchy: Hierarchy<'id>,
        store: Self::Store<'id>,
        effects: Effects<'id, D, E>,
    ) -> Self::Output<'id, D, E>
    where
        Self: Sized,
        D: EffectDialect + 'id,
        E: 'id;
}

/// Panicking host implementation for `std::convert::Infallible`.
///
/// This exists mostly to make doctests and examples easier to write.
impl Host for std::convert::Infallible {
    type Store<'id> = Self;
    type Output<'id, D, E>
        = Self
    where
        D: EffectDialect + 'id,
        E: 'id;

    fn new_store<'id>(&self) -> Self::Store<'id> {
        panic!("acquired reference to std::convert::Infallible")
    }

    fn finalize_pass<'id, D, E>(
        &self,
        _hierarchy: Hierarchy<'id>,
        _store: Self::Store<'id>,
        _effects: Effects<'id, D, E>,
    ) -> Self::Output<'id, D, E>
    where
        D: EffectDialect + 'id,
        E: 'id,
    {
        self.new_store()
    }
}
