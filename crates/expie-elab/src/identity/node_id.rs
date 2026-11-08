//! Identity and indexing for nodes in a [`Hierarchy`].

use crate::identity::{
    Hierarchy, Node,
    index::{ErasedIndex, ScopedIndex},
};

#[must_use = "uniquely identifies a node in its hierarchy"]
#[derive(
    Clone, Copy, derive_more::Debug, derive_more::Display, PartialEq, Eq, Hash, PartialOrd, Ord,
)]
#[display("id:{}", _0)]
#[debug("id:{:?}", _0)]
pub struct NodeId<'id>(ScopedIndex<'id>);

impl NodeId<'_> {
    /// Returns `true` if this is the first node allocated in the
    /// [`Hierarchy`].
    #[must_use = "result of an equality check"]
    pub const fn is_initial(self) -> bool {
        let Self(index_id) = self;
        index_id.is_initial()
    }

    /// Coerce `self` into an [`ErasedNodeId`], erasing the
    /// [invariant lifetime].
    ///
    /// This allows the ID to be used outside the bounds of the lifetime guard.
    /// This also means that it is no longer possible to guarantee that the ID
    /// accurately represents the original node it referenced.
    ///
    /// [invariant lifetime]: generativity::Guard
    pub const fn erased(self) -> ErasedNodeId {
        let Self(index_id) = self;
        ErasedNodeId(index_id.erased())
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
    pub fn push_node(&mut self, parent: Option<NodeId<'id>>) -> NodeId<'id> {
        let node_id = NodeId(self.nodes.push(Node {
            parent,
            children: Vec::new(),
        }));
        if let Some(NodeId(parent_id)) = parent {
            self.nodes.get_mut(parent_id).children.push(node_id);
        }
        node_id
    }

    /// Get a reference to a [`Node`] by its [`NodeId`].
    ///
    /// Node IDs can be retrieved only from the [`Self::push_node`] operation
    /// that added them to the [`Hierarchy`].
    #[must_use = "returns a reference to the looked-up node"]
    pub fn node(&self, node_id: NodeId<'id>) -> &Node<'id> {
        let NodeId(index_id) = node_id;
        self.nodes.get(index_id)
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
    #[must_use]
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
pub struct ErasedNodeId(ErasedIndex);

impl ErasedNodeId {
    /// Returns the actual underlying index of the [`ErasedNodeId`].
    #[must_use = "getter"]
    pub const fn raw_value(self) -> usize {
        let Self(index) = self;
        index.raw_value()
    }
}
