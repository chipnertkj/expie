use expie_elab::{EffectDialect, ScopeCx, Term};

use crate::{
    expr::{Num, Op},
    host::ExprHost,
};

/// Pure value term.
/// We need a newtype wrapper here for [`Term`] due to the orphan rule.
pub struct Value(pub Num);

/// [`Term`] impl for [`Value`].
impl<'id, D, E> Term<'id, ExprHost, D, E> for Value {
    fn lower(self, cx: &mut ScopeCx<'id, '_, ExprHost, D, E>)
    where
        D: EffectDialect,
    {
        cx.register(None, |child_cx| {
            child_cx.store_mut().push(Op::Value(self.0));
        });
    }
}
