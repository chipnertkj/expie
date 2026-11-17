use expie_elab::{PassCx, Term};

use crate::{
    expr::{Num, Op},
    host::ExprHost,
    reactive::{Reactive, ReactiveBinding, ReactiveEffect},
};

/// Reactively tracked input value term.
#[derive(Debug, Clone, Copy)]
pub struct Track(Num);

impl<'id> Term<'id, ExprHost, Reactive, ReactiveEffect> for Track {
    fn lower(self, cx: &mut PassCx<'id, '_, '_, ExprHost, Reactive, ReactiveEffect>) {
        cx.register(None, |cx, node| {
            let Self(value) = self;
            // Register as tracked value.
            let store = cx.store_mut();
            store.push(Op::Value(value));
            cx.push_effect(ReactiveBinding::ValueChange {
                node,
                effect: ReactiveEffect::Invalidate,
            });
        });
    }
}

/// Tracked input value under the [`Reactive`] dialect.
///
/// [`Reactive`]: crate::reactive::Reactive
#[must_use = "term constructor"]
pub const fn track(value: Num) -> Track {
    Track(value)
}
