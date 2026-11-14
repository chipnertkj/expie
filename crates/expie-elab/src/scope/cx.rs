//! A context for mutating a [`Scope`] under a specific node.

use crate::{Effects, Host, Marker, NodeId, Scope, effect::EffectDialect};

/// A view of [`Scope`](Scope), targetting an existing node in the hierarchy.
///
/// [`Self::target_node`] is none if the current node is a *root node* (i.e.
/// does not have a parent). When using [`scope`], the only situation in which
/// this happens is outside the context bounds where the first entity is
/// registered. See [`Self::register`] for more details.
///
/// The scope context additionally provides `push` / `extend` access to the
/// [`Effects`] associated with the scope.
///
/// [`scope`]: crate::scope::HostScope::scope
#[derive(Debug)]
pub struct ScopeCx<'id, 'scope, H: Host, D: EffectDialect, E> {
    /// A view of the [`Scope`] this context enables mutation of.
    scope: &'scope mut Scope<'id, H>,
    /// The node under which the next entity will be registered.
    target_node: Option<NodeId<'id>>,
    /// Mutable view over the [`Effects`] associated with the scope.
    effects: &'scope mut Effects<D, E>,
}

impl<'id, 'scope, H: Host, D: EffectDialect, E> ScopeCx<'id, 'scope, H, D, E> {
    /// Create a new [`ScopeCx`] for a fresh [`Scope`], with the `target_node`
    /// field set to [`None`].
    ///
    /// The caller must provide a mutable reference to a newly created
    /// [`Effects`] collection.
    ///
    /// Calling [`Self::register`] on a fresh context will register the new node
    /// as a *root node* (one without a parent).
    pub(in crate::scope) const fn new(
        scope: &'scope mut Scope<'id, H>,
        effects: &'scope mut Effects<D, E>,
    ) -> Self {
        Self {
            scope,
            target_node: None,
            effects,
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
        F: for<'rb> FnOnce(&mut ScopeCx<'id, 'rb, H, D, E>) -> T,
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
            // This explicit reborrow is intentional to underline the
            // importance of this operation.
            // It shortens the lifetime from `'scope` to `'rb`!
            // (Autoreborrow would do the same here.)
            scope: &mut *self.scope,
            target_node: Some(id),
            effects: self.effects,
        };
        // Call body with child scope instead of self.
        f(&mut new_cx)
    }

    /// Map the current effect type `E` to a new effect type `Inner` according
    /// to the [`EffectDialect`] `D` – this state holds only inside `body`.
    ///
    /// A new [`Effects`] collection is created for the effect type `Inner` for
    /// the duration of the function call. The `body` function is called
    /// with a new [`ScopeCx`], which reborrows [`Self::scope`] and borrows the
    /// newly instantiated effects collection.
    ///
    /// Effects pushed to the inner scope are later mapped to the outer
    /// effect type `E` via the `map` function, according to the rules specified
    /// by the used [`EffectDialect`] `D`.
    pub fn map_effect<Inner, R, Body, Map>(&mut self, map: Map, body: Body) -> R
    where
        Body: for<'rb> FnOnce(&mut ScopeCx<'id, 'rb, H, D, Inner>) -> R,
        Map: Clone + Fn(Inner) -> E,
    {
        let mut inner = Effects::<D, Inner>::new();
        let result = {
            let mut inner_cx = ScopeCx {
                // This explicit reborrow is intentional to underline the
                // importance of this operation.
                // It shortens the lifetime from `'scope` to `'rb`!
                // (Autoreborrow would do the same here.)
                scope: &mut *self.scope,
                target_node: self.target_node,
                effects: &mut inner,
            };
            body(&mut inner_cx)
        };
        self.effects.extend(inner.map_effect(map));
        result
    }

    /// Push an effect [`Binding`] to the [`Effects`] collection associated with
    /// the scope.
    ///
    /// [`Binding`]: EffectDialect::Binding
    pub fn push_effect(&mut self, effect: D::Binding<E>) {
        self.effects.push(effect);
    }

    /// Extend the [`Effects`] collection associated with the scope with the
    /// provided [`Binding`] iterator.
    ///
    /// [`Binding`]: EffectDialect::Binding
    pub fn extend_effects(&mut self, effects: impl IntoIterator<Item = D::Binding<E>>) {
        self.effects.extend(effects);
    }

    /// Get the current `target_node` of the [`ScopeCx`].
    ///
    /// If [`None`], the next entity registered via [`Self::register`] will be
    /// considered a *root node* in the parent-child hierarchy.
    #[must_use = "getter"]
    pub const fn target_node(&self) -> Option<NodeId<'id>> {
        self.target_node
    }

    /// Get a direct reference to the [`Host::Store`] of the [`Scope`].
    #[must_use = "getter"]
    pub const fn store(&self) -> &H::Store<'id> {
        &self.scope.store
    }

    /// Get a mutable direct reference to the [`Host::Store`] of the [`Scope`].
    #[must_use = "getter"]
    pub const fn store_mut(&mut self) -> &mut H::Store<'id> {
        &mut self.scope.store
    }
}
