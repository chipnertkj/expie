//! A context for mutating a [`Scope`] under a specific node.

use crate::{Host, Marker, NodeId, Scope};

/// A view of [`Scope`], targetting an existing node in the hierarchy.
///
/// [`Self::target_node`] is none if the current node is a *root node* (i.e.
/// does not have a parent). When using [`scope`], the only situation in which
/// this happens is outside the context bounds where the first entity is
/// registered. See [`Self::register`] for more details.
///
/// [`scope`]: crate::scope::HostScope::scope
#[derive(Debug)]
pub struct ScopeCx<'id, 'scope, H: Host + ?Sized> {
    /// A view of the [`Scope`] this context enables mutation of.
    scope: &'scope mut Scope<'id, H>,
    /// The node under which the next entity will be registered.
    target_node: Option<NodeId<'id>>,
}

impl<'id, 'scope, H: Host> ScopeCx<'id, 'scope, H> {
    /// Create a new [`ScopeCx`] for a fresh [`Scope`], with the `target_node`
    /// field set to [`None`].
    ///
    /// Calling [`Self::register`] on this context will register a new node as a
    /// *root node* (one without a parent).
    pub(in crate::scope) const fn new(scope: &'scope mut Scope<'id, H>) -> Self {
        Self {
            scope,
            target_node: None,
        }
    }

    /// Register a new entity under the current `target_node`.
    ///
    /// If the `marker` is [`Some`], the node will be [marked] with the given
    /// [`Marker`]. In either case, this method registers a new [`NodeId`].
    ///
    /// [marked]: crate::Hierarchy::push_node_marked
    ///
    ///
    /// If [`Self::target_node`] is [`Some`], the new node will be registered
    /// under it (as a child).
    ///
    /// This method takes a function `f`, which receives a newly constructed
    /// [`ScopeCx`], which reborrows [`Self::scope`] for the duration of the
    /// function call, with the `target_node` of the new context set to the
    /// newly registered node.
    pub fn register<T, F>(&mut self, marker: Option<Marker<'id>>, f: F) -> T
    where
        F: for<'rb> FnOnce(&mut ScopeCx<'id, 'rb, H>) -> T,
    {
        // Register and optionally mark a new node.
        // The node will be positioned under the current target node.
        let hierarchy = &mut self.scope.hierarchy;
        let id = if let Some(marker) = marker {
            hierarchy.push_node_marked(self.target_node, marker)
        } else {
            hierarchy.push_node(self.target_node)
        };
        // Create a scope view under the new node via reborrow.
        let mut new_cx = ScopeCx {
            // This dereference is intentional and *very important*.
            // It shortens the lifetime of 'scope to 'rb via reborrowing!
            scope: &mut *self.scope,
            target_node: Some(id),
        };
        // Call body with child scope instead of self.
        f(&mut new_cx)
    }

    /// Get the current `target_node` of the [`ScopeCx`].
    ///
    /// If [`None`], the next entity registered via [`Self::register`] will be
    /// considered a *root node* in the parent-child hierarchy.
    #[must_use = "getter"]
    pub const fn target_node(&self) -> Option<NodeId<'id>> {
        self.target_node
    }
}
