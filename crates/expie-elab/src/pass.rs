//! Transient lifetime-bound state for mutation by [`Term`]s during elaboration.
//!
//! [`Term`]: crate::Term

use crate::{Effects, Hierarchy, Host, Marker, MarkerRef, effect::EffectDialect};

mod cx;

pub use self::cx::PassCx;

/// State provided to [`Term`]s during [`Term::lower`].
///
/// [`Term`]: crate::Term
/// [`Term::lower`]: crate::Term::lower
///
/// Use [`HostScope::scope`] to create a new instance of this type.
/// Then, lower a term on this pass by opening a [`Pass::cx`].
///
/// Terms are to mutate the store owned by the `Pass` in the process of
/// elaboration (lowering). Once all terms have been lowered, the pass is
/// finalized by calling [`Pass::finalize`], resulting in a host-defined
/// [`Host::Output`].
#[derive(derive_more::Debug)]
pub struct Pass<'id, 'host, H: Host> {
    /// Identity allocations for terms in this pass.
    /// Note that entity count may not necessarily match the number of terms
    /// [lowered] here.
    ///
    /// [lowered]: crate::Term::lower
    hierarchy: Hierarchy<'id>,
    /// Target store for [`Term::lower`] in this pass.
    ///
    /// [`Term::lower`]: crate::Term::lower
    #[debug("some_store")]
    store: H::Store<'id>,
    /// [`Host`] that started this pass.
    #[debug(ignore)]
    host: &'host H,
}

impl<'id, 'host, H: Host> Pass<'id, 'host, H> {
    /// Create a new pass for a given [`Host`], bound to an invariant
    /// [generative lifetime].
    ///
    /// [generative lifetime]: generativity::Id
    fn new(host: &'host H, gen_id: generativity::Id<'id>) -> Self {
        let hierarchy = Hierarchy::new(gen_id);
        let store = host.new_store();
        Self {
            hierarchy,
            store,
            host,
        }
    }

    /// Create a new [`Marker`] for this pass.
    ///
    /// Returns a `Marker` + [`MarkerRef`] tuple pair.
    /// Marking a node is only useful if you plan on referencing it later, so
    /// returning both saves you from having to [`Marker::downgrade`] manually.
    #[must_use = "marker must be consumed by target entity"]
    pub fn marker(&mut self) -> (Marker<'id>, MarkerRef<'id>) {
        let marker = self.hierarchy.marker();
        let marker_ref = *marker.downgrade();
        (marker, marker_ref)
    }

    /// Borrow `self` together with `effects` as a root [`PassCx`] and run `f`.
    pub fn cx<'pass, D, E, F, O>(&'pass mut self, effects: &'pass mut Effects<'id, D, E>, f: F) -> O
    where
        F: FnOnce(PassCx<'id, 'host, 'pass, H, D, E>) -> O,
        D: EffectDialect + 'id,
        E: 'id,
    {
        f(PassCx::new(self, effects))
    }

    /// Consume this pass and `effects`, producing the host
    /// [`Output`](Host::Output).
    pub fn finalize<D, E>(self, effects: Effects<'id, D, E>) -> H::Output<'id, D, E>
    where
        D: EffectDialect,
        E: 'id,
    {
        let Pass {
            hierarchy,
            store,
            host,
        } = self;
        host.finalize_pass(hierarchy, store, effects)
    }
}

/// An extension trait for [`Host`] that provides a method, [`Self::scope`], for
/// writing code scoped by an invariant [generative lifetime guard].
///
/// [generative lifetime guard]: generativity::make_guard!
pub trait HostScope: Host {
    /// Create a new [`Pass`], bounded by an invariant
    /// [generative lifetime guard], as well as an [`Effects`] collection, then
    /// execute `f` on both.
    ///
    /// [generative lifetime guard]: generativity::make_guard!
    ///
    /// This process is referred to as "opening a scope".
    ///
    /// # Usage
    ///
    /// This method takes a closure `f` that receives both `Pass` and `Effects`
    /// as arguments. The expected procedure for `f` is to:
    /// 1. Construct a [`Term`] structure.
    /// 2. Create a [`PassCx`] via [`Pass::cx`].
    /// 3. [Lower] the constructed term in the `PassCx`.
    /// 4. After `cx` is done, finalize the `Pass` via [`Pass::finalize`].
    /// 5. Process the resulting [`Host::Output`], potentially by
    ///    lifetime-erasing values and returning output data `O`.
    ///
    /// [lower]: crate::Term::lower
    /// [`Term`]: crate::Term
    ///
    /// The return value `O` cannot reference `'id`.
    /// This means that `f` cannot smuggle out any data that is valid
    /// only within this `scope` call, with the exception of explicitly
    /// *lifetime-erased* data – e.g. node identifiers provide
    /// [`NodeId::erased`] for this purpose.
    ///
    /// [`NodeId::erased`]: crate::NodeId::erased
    fn scope<'host, D, E, F, O>(&'host self, f: F) -> O
    where
        F: for<'id> FnOnce(Pass<'id, 'host, Self>, Effects<'id, D, E>) -> O,
        D: EffectDialect;
}

/// Blanket implementation for all [`Host`]s.
impl<H: Host> HostScope for H {
    fn scope<'host, D, E, F, O>(&'host self, f: F) -> O
    where
        F: for<'id> FnOnce(Pass<'id, 'host, Self>, Effects<'id, D, E>) -> O,
        D: EffectDialect,
    {
        generativity::make_guard!(id_guard);
        // Initialize scope state.
        // The caller lowers via [`Pass::cx`] and may [`Pass::finalize`] before
        // returning.
        let pass = Pass::new(self, id_guard.into());
        let effects = Effects::<D, E>::new();
        f(pass, effects)
    }
}
