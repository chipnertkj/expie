use expie_elab::{EffectDialect, MarkerRef, ScopeCx, Term};

use crate::{expr::Op, host::ExprHost};

/// Reference a marked entity term via its [`MarkerRef`].
pub struct Var<'id>(pub MarkerRef<'id>);

/// [`Term`] impl for [`Var`].
impl<'id, D, E> Term<'id, ExprHost, D, E> for Var<'id> {
    fn lower(self, cx: &mut ScopeCx<'id, '_, ExprHost, D, E>)
    where
        D: EffectDialect,
    {
        cx.register(None, |child_cx| {
            let Self(marker) = self;
            child_cx.store_mut().push(Op::Var(marker));
        });
    }
}
