//! Abstract Syntax Tree (AST) term definition.
//!
//! # Responsibility
//!
//! A term acts as the translation instrument for the *sources of information*
//! (mentioned in [`Term::lower`]'s documentation) into data specific to a
//! [`Host::Store`].
//!
//! Because of the definition of `lower`, *lowering* can be thought of
//! as a form of *elaboration* to the scope, on what a term's intent is. Both
//! "terms" (in the sense of *terminology*, i.e. elaboration and
//! lowering) refer to the same action, but express very different
//! ideas.
//!
//! Elaboration is the process of building up an understanding of the
//! desired result, by filling in details to achieve an outcome. Lowering,
//! on the other hand, strips away semantic details and translates them into
//! instructions for the next step in the overall process.

use crate::{Host, ScopeCx, for_each_tuple_12};

/// A term is a unit of data in an Abstract Syntax Tree (AST).
///
/// `expie` does not specify the actual AST in use.
/// This architecture instead allows us to represent arbitrary declarative trees
/// as compile-time syntax, which can be lowered down into runtime details.
///
/// Declaring a term only requires that it knows how to [`lower`] itself down
/// into the appropriate implementation details specific to a particular
/// [`Host`].
///
/// A term is supposed to work as an abstraction of a domain-specific feature.
/// Its fields / variants and methods should act as an interface in
/// communicating intent. `lower` provides the glue between that ideal and the
/// reality of the software components a term integrates with.
///
/// [`lower`]: Self::lower
// TODO: explain `M` once messages are implemented.
pub trait Term<'id, H: Host + ?Sized, M> {
    /// Lower this term inside a [`ScopeCx`].
    ///
    /// *Lowering* is the process of mutating the [`Scope`] (which the scope
    /// context borrows) by providing it with additional information.
    ///
    /// [`Scope`]: crate::Scope
    ///
    /// A term writes information to the [`Host::Store`] (owned by the scope)
    /// via methods exposed on the scope context.
    ///
    /// Conceptually, this information should come from:
    ///   1. The occurrence of this term in the lowering process.
    ///   2. The data contained within this term's value.
    ///
    /// # Identity and hierarchy
    ///
    /// In order for lowered data to maintain identity (i.e. refer back to the
    /// term it originates from), the scope provides identity registration
    /// via [`ScopeCx::register`]. This allocates a unique [`NodeId`] and opens
    /// a new [`ScopeCx`], with the new `Node` set as the *current target*.
    ///
    /// [`NodeId`]: crate::NodeId
    ///
    /// The newly constructed context is available only within a closure, which
    /// is passed to `register` by the caller.
    ///
    /// This pattern allows for constructing hierarchies of nodes, as any node
    /// registered using that inner context will be treated as a child of the
    /// current target node. This parent-child tree hierarchy may be later
    /// queried by the [`Host`] to resolve relationships.
    ///
    /// [`Host`]: crate::Host
    ///
    /// # Term kinds
    ///
    /// Note that registration is completely optional. A term may contribute
    /// information without registering as an individual entity –
    /// you may refer to this pattern as a **"virtual term"**.
    ///
    /// A term that doesn't register itself, but lowers multiple terms under the
    /// same target node is analogously a **"sequence term"** – it expands to a
    /// series of terms that may register their own identities. They get
    /// lowered in the order defined by the sequence term. An
    /// example of this is the term implementation for tuples.
    ///
    /// It is possible to implement a term that registers multiple identities
    /// for itself. This is also valid – it declares multiple nodes, each a
    /// target for different child edges. In such cases, it is up to the
    /// host and that term to correctly orchestrate and interpret those
    /// relationships. This pattern may be referred to as a **"polysemous
    /// term"**.
    fn lower(self, cx: &mut ScopeCx<'id, '_, H>);
}

/// Implement sequence [`Term`] for a tuple of terms.
macro_rules! impl_tuple_term {
    ($($name:ident),* $(,)?) => {
        impl<'id, HOST, MESSAGE, $($name),*> Term<'id, HOST, MESSAGE> for ($($name),*,)
        where
            HOST: Host + ?Sized,
            $($name: Term<'id, HOST, MESSAGE>),*
        {
            fn lower(self, cx: &mut ScopeCx<'id, '_, HOST>) {
                #[allow(non_snake_case, reason = "macro expansion")]
                let ($($name),*,) = self;
                $($name.lower(cx);)*
            }
        }
    };
}

for_each_tuple_12!(impl_tuple_term);
