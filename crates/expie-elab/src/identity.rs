//! Identification and hierarchical relationships between arbitrary entities.

mod index;
mod marker;
mod node_id;

use self::index::ScopedIndexVec;
pub use self::{
    marker::{Marker, MarkerRef},
    node_id::NodeId,
};

/// A relationship representing a uniquely positioned entity within a
/// [`Hierarchy`].
///
/// A node represents an arbitrary entity by positioning it between another node
/// – its parent – and its children, which are also nodes.
///
/// This relationship is represented purely by [`NodeId`]s created by
/// [`Hierarchy::push_node`] – the method used to register nodes.
#[derive(Debug, Clone)]
struct Node<'id> {
    /// The parent of this entity, if any.
    parent: Option<NodeId<'id>>,
    /// The children of this entity, in allocation order.
    ///
    /// See [`Hierarchy::push_node`] for more information.
    children: Vec<NodeId<'id>>,
}

/// A collection of [`Node`]s, indexed by [`NodeId`].
///
/// This structure, via relationships represented by [`Node`]s, represents a
/// tree of arbitrary entities, each uniquely identified by a [`NodeId`].
///
/// It also establishes a referencing scheme via [`Marker`]s, which can be used
/// to refer to nodes anonymously, without knowing their [`NodeId`]s upfront.
///
/// All data is scoped to the [invariant lifetime] `'id`, and any indices
/// constrained by this unique lifetime are guaranteed to accurately reference
/// the original entity they were allocated for.
///
/// This structure is exclusively additive and pushed nodes can never be removed
/// from it.
///
/// [invariant lifetime]: generativity::Guard
#[derive(Debug, Clone)]
pub struct Hierarchy<'id> {
    /// The [`Node`]s registered in this hierarchy.
    nodes: ScopedIndexVec<'id, Node<'id>>,
    /// The [`Marker`]s registered in this hierarchy.
    ///
    /// A marker slot (i.e. index) that holds [`None`] was registered, but its
    /// entity was never tagged. Entity [`Node`]s must be registered with
    /// [`Hierarchy::push_node_marked`] in order to be considered tagged.
    markers: ScopedIndexVec<'id, Option<NodeId<'id>>>,
}

impl<'id> Hierarchy<'id> {
    /// Create an empty [`Hierarchy`] representing a relationship tree for
    /// arbitrary entities.
    ///
    /// The [invariant lifetime] handle `gen_id` passed here will be used when
    /// pushing new nodes to the hierarchy. Its lifetime `'id` will ensure that
    /// the indices of the nodes are valid for that lifetime.
    ///
    /// [invariant lifetime]: generativity::Guard
    #[must_use = "newly allocated structure"]
    pub const fn new(gen_id: generativity::Id<'id>) -> Self {
        Self {
            nodes: ScopedIndexVec::new(gen_id),
            markers: ScopedIndexVec::new(gen_id),
        }
    }

    /// Returns the number of [`Node`]s registered in the hierarchy.
    #[must_use = "length check"]
    pub const fn nodes_len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns `true` if the hierarchy is represented by no [`Node`]s.
    #[must_use = "length check"]
    pub const fn is_nodes_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Get the optional parent [`NodeId`] of another [`Node`].
    ///
    /// Returns [`None`] if the node is a *root node*.
    #[must_use = "returns parent node ID"]
    pub fn parent_of(&self, node_id: NodeId<'id>) -> Option<NodeId<'id>> {
        self.node(node_id).parent
    }

    /// Iterate the children of a [`Node`], in their allocation order.
    ///
    /// See [`Hierarchy::push_node`] for more information.
    #[must_use = "iterates over children"]
    pub fn children_of(&self, node_id: NodeId<'id>) -> impl Iterator<Item = NodeId<'id>> + '_ {
        self.node(node_id).children.iter().copied()
    }
}
