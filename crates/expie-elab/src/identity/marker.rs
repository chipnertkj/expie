//! [`Marker`]s are used to tag entity occurences in a [`Hierarchy`].

use crate::{Hierarchy, NodeId, identity::index::ScopedIndex};

/// Markers are unique, anonymous tags that reference [`Node`]s in a
/// [`Hierarchy`].
///
/// This tag is unique, as in:
///   1. It cannot be cloned;
///   2. The provider of this value must ensure that a marker can only be
///      obtained directly from the entity it references.
///
/// This definition can be rephrased in *Capability / Handle* terms:
///   1. A `Marker` represents the exclusive right to define or register a
///      single *tagged* entity.
///   2. Consuming the `Marker` binds that identity permanently to exactly one
///      entity node.
///
/// To achieve the semantics targetted here, the provider is to ensure
/// construction is aligned with the above, and that the marker is consumed upon
/// tagging an entity. Since it isn't cloneable, ownership rules guarantee
/// correctness thanks to static analysis.
///
/// A marker can be constructed before the entity node is registered in the
/// hierarchy.
/// This has two implications:
///   - It is possible to mark entities that aren't registered yet, if we intend
///     to register them later;
///   - It is possible to create markers that end up never resolving to an
///     existing node, if a node tagged with this marker is never registered.
///
/// While the latter introduces a risk of dangling references, the first allows
/// for an inversion of data flow in expressing relationships between entities
/// in a hierarchy. This is the accepted trade-off.
///
/// [`Hierarchy`]: crate::identity::Hierarchy
/// [`Node`]: crate::identity::Node
#[derive(derive_more::Debug, derive_more::Display, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[display("marker:{}", _0)]
#[debug("marker:{:?}", _0)]
pub struct Marker<'id>(
    /// Identity information for the marker.
    MarkerRef<'id>,
);

impl<'id> Marker<'id> {
    /// Extract a [`MarkerRef`] from a [`Marker`] without consuming it.
    ///
    /// This operation is valid, as the marker remains the authorative source of
    /// truth for the tag, while the `MarkerRef` only offers a view on it,
    /// according to rules defining the tagging relationship (i.e. the
    /// definition of a `Marker`).
    #[must_use = "getter"]
    pub const fn downgrade(&self) -> &MarkerRef<'id> {
        let Marker(marker_ref) = self;
        marker_ref
    }
}

/// A copyable reference to a unique, anonymous tag of an entity in a
/// [`Hierarchy`]. This tagging relationship is represented by a [`Marker`].
///
/// [`Hierarchy`]: crate::identity::Hierarchy
#[derive(
    Clone, Copy, derive_more::Debug, derive_more::Display, Eq, Hash, Ord, PartialEq, PartialOrd,
)]
#[display("ref:{}", _0)]
#[debug("ref:{:?}", _0)]
pub struct MarkerRef<'id>(
    /// The [`ScopedIndex`] allocated for the [`Marker`] which this reference
    /// represents.
    ScopedIndex<'id>,
);

impl<'id> MarkerRef<'id> {
    /// Extract a [`ScopedIndex`] from a [`MarkerRef`].
    #[must_use = "extraction of an index value"]
    const fn to_index(self) -> ScopedIndex<'id> {
        let MarkerRef(scoped_index) = self;
        scoped_index
    }
}

impl<'id> From<&Marker<'id>> for MarkerRef<'id> {
    fn from(value: &Marker<'id>) -> Self {
        *value.downgrade()
    }
}

impl<'id> AsRef<MarkerRef<'id>> for Marker<'id> {
    fn as_ref(&self) -> &MarkerRef<'id> {
        self.downgrade()
    }
}

impl<'id> Hierarchy<'id> {
    /// Push a new [`Node`] to the hierarchy (via [`Hierarchy::push_node`]), and
    /// tag it with a [`Marker`].
    ///
    /// This operation consumes the [`Marker`], which is required to follow the
    /// rules of the tagging relationship (i.e. the definition of a `Marker`).
    ///
    /// This is the only valid way to tag a node with a marker.
    ///
    /// [`Node`]: crate::identity::Node
    #[must_use = "newly allocated identifier, you probably want to keep this"]
    // TODO: tighten this vis to future cx module
    pub(crate) fn push_node_marked(
        &mut self,
        parent: Option<NodeId<'id>>,
        marker: Marker<'id>,
    ) -> NodeId<'id> {
        let node_id = self.push_node(parent);
        {
            // Move the marker into the block to ensure we consume it here.
            let marker: Marker<'_> = marker;
            // Since we have succesfully obtained the marker (types match),
            // we can safely tag the node with the marker.
            let scoped_index = marker.downgrade().to_index();
            *self.markers.get_mut(scoped_index) = Some(node_id);
            // The marker is consumed in the process.
            let _: Marker<'_> = marker;
        }
        node_id
    }

    /// Get the [`NodeId`] of the [`Node`] tagged with a [`Marker`].
    ///
    /// Due to the way marker tagging works (see [`Marker`] documentation), it
    /// is possible for an allocated marker to act as a dangling reference –
    /// this happens if it is never used to tag a registered node (with
    /// [`Hierarchy::push_node_marked`]). In that situation, [`None`] is
    /// returned instead.
    ///
    /// [`Node`]: crate::identity::Node
    #[must_use = "getter"]
    pub fn marked_node(&self, marker_ref: MarkerRef<'id>) -> Option<NodeId<'id>> {
        let scoped_index = marker_ref.to_index();
        *self.markers.get(scoped_index)
    }
}
