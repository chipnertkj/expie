//! Identification and hierarchical relationships between arbitrary entities.

mod index;
mod node_id;

use self::index::ScopedIndexVec;
pub use self::node_id::NodeId;

/// A relationship representing a uniquely positioned entity within a
/// [`Hierarchy`].
///
/// A node represents an arbitrary entity by positioning it between another node
/// – its parent – and its children, which are also nodes.
///
/// This relationship is represented purely by [`NodeId`]s created by
/// [`Hierarchy::push_node`] – the method used to register nodes.
#[must_use = "represents a hierarchical relationship"]
#[derive(Debug, Clone)]
pub struct Node<'id> {
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
/// All data is scoped to the [invariant lifetime] `'id`, and any indices
/// constrained by this unique lifetime are guaranteed to accurately reference
/// the original entity they were allocated for.
///
/// This structure is exclusively additive and pushed nodes can never be removed
/// from it.
///
/// [invariant lifetime]: generativity::Guard
#[must_use = "represents a hierarchical structure of entities"]
#[derive(Debug, Clone)]
pub struct Hierarchy<'id> {
    /// The nodes registered in this hierarchy.
    /// This vector is to only be pushed to, and never removed from.
    nodes: ScopedIndexVec<'id, Node<'id>>,
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
    pub const fn new(gen_id: generativity::Id<'id>) -> Self {
        Self {
            nodes: ScopedIndexVec::new(gen_id),
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
