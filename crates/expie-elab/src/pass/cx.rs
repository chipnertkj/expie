//! A context for mutating a [`Pass`] under a specific node.

use crate::{Effects, Host, Marker, NodeId, Pass, effect::EffectDialect};

/// A view of [`Pass`], targetting an existing node in the hierarchy.
///
/// [`Self::target_node`] is [`None`] if the current node is a *root node* (i.e.
/// does not have a parent).
/// The only time this is true is if this context is "fresh", i.e. just newly
/// created via [`Pass::cx`].
///
/// The pass context additionally provides `push` / `extend` access to the
/// [`Effects`] associated with the [`scope`] call.
///
/// [`scope`]: crate::HostScope::scope
pub struct PassCx<'id, 'host, 'pass, H: Host, D: EffectDialect, E> {
    /// A view of the [`Pass`] this context enables mutation of.
    pass: &'pass mut Pass<'id, 'host, H>,
    /// The node under which the next entity will be registered.
    target_node: Option<NodeId<'id>>,
    /// Mutable view over the [`Effects`] associated with the scope.
    effects: &'pass mut Effects<'id, D, E>,
}

impl<'id, H, D, E> std::fmt::Debug for PassCx<'id, '_, '_, H, D, E>
where
    D::Binding<'id, E>: std::fmt::Debug,
    H: Host,
    D: EffectDialect,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PassCx")
            .field("pass", &self.pass)
            .field("target_node", &self.target_node)
            .field("effects", &self.effects)
            .finish()
    }
}

impl<'id, 'host, 'pass, H: Host, D: EffectDialect, E> PassCx<'id, 'host, 'pass, H, D, E> {
    /// Create a new [`PassCx`] for a fresh [`Pass`], with the `target_node`
    /// field set to [`None`].
    ///
    /// The caller must provide a mutable reference to a newly created
    /// [`Effects`] collection.
    ///
    /// Calling [`Self::register`] on a fresh context will register the new node
    /// as a *root node* (one without a parent).
    pub(in crate::pass) const fn new(
        pass: &'pass mut Pass<'id, 'host, H>,
        effects: &'pass mut Effects<'id, D, E>,
    ) -> Self {
        Self {
            pass,
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
    /// If [`Self::target_node`] is [`Some`], the new node will be registered
    /// under it (as a child).
    ///
    /// This method takes a function `f`. It receives a newly constructed
    /// [`PassCx`], which reborrows the original context's pass for the duration
    /// of the function call, with the `target_node` of the new context set
    /// to the newly registered node.
    /// `f` additionally receives the newly registered node's [`NodeId`] for
    /// convenience. You may discard it if it's of no use – descendants may
    /// still call [`Self::target_node`] to get it.
    pub fn register<T, F>(&mut self, marker: Option<Marker<'id>>, f: F) -> T
    where
        F: for<'rb> FnOnce(&mut PassCx<'id, 'host, 'rb, H, D, E>, NodeId<'id>) -> T,
    {
        // Register and optionally mark a new node.
        // The node will be positioned under the current target node.
        let hierarchy = &mut self.pass.hierarchy;
        let id = if let Some(marker) = marker {
            hierarchy.push_node_marked(self.target_node, marker)
        } else {
            hierarchy.push_node(self.target_node)
        };
        // Create a scope view under the new node via reborrow.
        let mut new_cx = PassCx {
            // This explicit reborrow is intentional to underline the
            // importance of this operation.
            // It shortens the lifetime from `'pass` to `'rb`!
            // (Autoreborrow would do the same here.)
            pass: &mut *self.pass,
            target_node: Some(id),
            effects: &mut *self.effects,
        };
        // Call body with child scope instead of self.
        f(&mut new_cx, id)
    }

    /// Map the current effect type `E` to a new effect type `Inner` according
    /// to the [`EffectDialect`] `D` – this state holds only inside `body`.
    ///
    /// A new [`Effects`] collection is created for the effect type `Inner` for
    /// the duration of the function call. The `body` function is called
    /// with a new [`PassCx`], which reborrows [`Self`]'s pass and borrows the
    /// newly instantiated effects collection.
    ///
    /// Effects pushed to the inner scope are later mapped to the outer
    /// effect type `E` via the `map` function, according to the rules specified
    /// by the used [`EffectDialect`] `D`.
    pub fn map_effect<Inner, R, Body, Map>(&mut self, map: Map, body: Body) -> R
    where
        Body: for<'rb> FnOnce(&mut PassCx<'id, 'host, 'rb, H, D, Inner>) -> R,
        Map: Clone + Fn(Inner) -> E,
    {
        let mut inner = Effects::<D, Inner>::new();
        let result = {
            let mut inner_cx = PassCx {
                // This explicit reborrow is intentional to underline the
                // importance of this operation.
                // It shortens the lifetime from `'pass` to `'rb`!
                // (Autoreborrow would do the same here.)
                pass: &mut *self.pass,
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
    pub fn push_effect(&mut self, effect: D::Binding<'id, E>) {
        self.effects.push(effect);
    }

    /// Extend the [`Effects`] collection associated with the scope with the
    /// provided [`Binding`] iterator.
    ///
    /// [`Binding`]: EffectDialect::Binding
    pub fn extend_effects(&mut self, effects: impl IntoIterator<Item = D::Binding<'id, E>>) {
        self.effects.extend(effects);
    }

    /// Get the current `target_node` of the [`PassCx`].
    ///
    /// If [`None`], the next entity registered via [`Self::register`] will be
    /// considered a *root node* in the parent-child hierarchy.
    #[must_use = "getter"]
    pub const fn target_node(&self) -> Option<NodeId<'id>> {
        self.target_node
    }

    /// Get a direct reference to the [`Host::Store`] of the [`Pass`].
    #[must_use = "getter"]
    pub const fn store(&self) -> &H::Store<'id> {
        &self.pass.store
    }

    /// Get a mutable direct reference to the [`Host::Store`] of the [`Pass`].
    #[must_use = "getter"]
    pub const fn store_mut(&mut self) -> &mut H::Store<'id> {
        &mut self.pass.store
    }
}
