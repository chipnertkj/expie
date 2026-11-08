//! Identity and indexing for nodes in a [`Hierarchy`].

use crate::identity::{
    Hierarchy, Node,
    index::{ErasedIndex, ScopedIndex},
};

/// A unique identifier for a [`Node`] in a [`Hierarchy`].
///
/// Allocating an identifier is done through the [`Hierarchy::push_node`]
/// method, which registers a node through its relationship to other nodes in
/// a tree.
#[derive(
    Clone, Copy, derive_more::Debug, derive_more::Display, PartialEq, Eq, Hash, PartialOrd, Ord,
)]
#[display("id:{}", _0)]
#[debug("id:{:?}", _0)]
pub struct NodeId<'id>(
    /// Identity information for the node.
    /// This index corresponds to the position of the node in the
    /// [`ScopedIndexVec`] inside of its [`Hierarchy`].
    ///
    /// [`ScopedIndexVec`]: crate::identity::index::ScopedIndexVec
    ScopedIndex<'id>,
);

impl<'id> NodeId<'id> {
    /// Returns `true` if this is the first node allocated in the
    /// [`Hierarchy`].
    #[must_use = "result of an equality check"]
    pub const fn is_initial(self) -> bool {
        let Self(index_id) = self;
        index_id.is_initial()
    }

    /// Convert `self` into an [`ErasedNodeId`], erasing the
    /// [invariant lifetime].
    ///
    /// This allows the ID to be used outside the bounds of the lifetime guard.
    /// This also means that it is no longer possible to guarantee that the ID
    /// accurately represents the original node it referenced.
    ///
    /// [invariant lifetime]: generativity::Guard
    #[must_use = "conversion to a lifetime-erased identifier"]
    pub const fn erased(self) -> ErasedNodeId {
        let Self(index_id) = self;
        ErasedNodeId(index_id.erased())
    }

    /// Extract a [`ScopedIndex`] from a [`NodeId`].
    #[must_use = "extraction of an index value"]
    const fn to_index(self) -> ScopedIndex<'id> {
        let Self(scoped_index) = self;
        scoped_index
    }
}

/// This `impl` block handles hierarchical operations which instantiate
/// [`NodeId`]s.
impl<'id> Hierarchy<'id> {
    /// Add a [`Node`] to the hierarchy, in order to represent an entity in the
    /// relationship tree.
    ///
    /// This effectively registers that entity as a node, by allocating a
    /// unique, compatible [`NodeId`] to represent it.
    ///
    /// If `parent` is set to [`None`], the node will be treated as a *root
    /// node*.
    #[must_use = "newly allocated identifier, you probably want to keep this"]
    // TODO: tighten this vis to future cx module
    pub(crate) fn push_node(&mut self, parent: Option<NodeId<'id>>) -> NodeId<'id> {
        // Push the node to the index vector.
        let node_index = self.nodes.push(Node {
            parent,
            children: Vec::new(),
        });
        // Convert the index to a node ID.
        let node_id = NodeId(node_index);
        // If the node has a parent, add it to the parent's children.
        if let Some(parent_id) = parent {
            self.node_mut(parent_id).children.push(node_id);
        }
        // Return the node ID.
        node_id
    }

    /// Get a reference to a [`Node`] by its [`NodeId`].
    #[must_use = "returns a reference to the looked-up node"]
    pub(in crate::identity) fn node(&self, node_id: NodeId<'id>) -> &Node<'id> {
        self.nodes.get(node_id.to_index())
    }

    /// Get a mutable reference to a [`Node`] by its [`NodeId`].
    #[must_use = "returns a mutable reference to the looked-up node"]
    fn node_mut(&mut self, node_id: NodeId<'id>) -> &mut Node<'id> {
        self.nodes.get_mut(node_id.to_index())
    }

    /// Iterate root [`Node`]s registered in the [`Hierarchy`], by their
    /// [`NodeId`]s, in allocation order.
    ///
    /// A *root node* is a [`Node`] with no parent (i.e. set to [`None`]).
    ///
    /// If there are multiple root nodes in the hierarchy, the structure
    /// represents a forest of entities instead of a single tree.
    ///
    /// See [`Hierarchy::push_node`] for information on allocation order.
    pub fn roots(&self) -> impl Iterator<Item = NodeId<'id>> + '_ {
        self.nodes
            .iter_indexed()
            .filter_map(|(index_id, node)| node.parent.is_none().then_some(NodeId(index_id)))
    }

    /// The first [`Node`] allocated in this [`Hierarchy`], if any.
    ///
    /// See [`Hierarchy::push_node`] for information on allocation order.
    #[must_use = "returns the first root node, if any"]
    pub fn main_root(&self) -> Option<NodeId<'id>> {
        self.nodes
            .iter_indexed()
            .next()
            .map(|(index_id, _)| NodeId(index_id))
    }
}

/// A [`NodeId`] that has had its [invariant lifetime] erased.
///
/// This allows the ID to be used outside the bounds of the lifetime guard.
/// This also means that it is no longer possible to guarantee that the ID
/// accurately represents the original node it referenced.
///
/// [invariant lifetime]: generativity::Guard
#[derive(derive_more::Debug, derive_more::Display, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[display("id:{}", _0)]
#[debug("{:?}", _0)]
pub struct ErasedNodeId(ErasedIndex);

impl ErasedNodeId {
    /// Returns the actual underlying index of the [`ErasedNodeId`].
    #[must_use = "getter"]
    pub const fn raw_value(self) -> usize {
        let Self(index) = self;
        index.raw_value()
    }
}
