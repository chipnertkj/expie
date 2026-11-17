use expie_elab::{EffectDialect, NodeId};

/// Possible side effects that may be caused by a reactive term.
#[derive(Clone, Copy, Debug)]
pub enum ReactiveEffect {
    Invalidate,
}

/// Bindings that may trigger reactive side effects.
#[derive(Clone, Copy, Debug)]
pub enum ReactiveBinding<'id, E> {
    ValueChange { node: NodeId<'id>, effect: E },
}

/// Reactive term effect dialect.
#[derive(Debug, Default, Clone, Copy)]
pub struct Reactive;

impl EffectDialect for Reactive {
    type Binding<'id, E> = ReactiveBinding<'id, E>;

    fn map_effect<Old, New, F>(binding: Self::Binding<'_, Old>, map: F) -> Self::Binding<'_, New>
    where
        F: Fn(Old) -> New,
    {
        match binding {
            ReactiveBinding::ValueChange { node, effect } => ReactiveBinding::ValueChange {
                node,
                effect: map(effect),
            },
        }
    }
}
