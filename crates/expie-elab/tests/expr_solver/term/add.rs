use expie_elab::{EffectDialect, ScopeCx, Term};

use crate::{expr::Op, host::ExprHost};

/// Addition term.
pub struct Add<L, R>(pub L, pub R);

/// [`Term`] impl for [`Add`].
impl<'id, L, R, D, E> Term<'id, ExprHost, D, E> for Add<L, R>
where
    L: Term<'id, ExprHost, D, E>,
    R: Term<'id, ExprHost, D, E>,
{
    fn lower(self, cx: &mut ScopeCx<'id, '_, ExprHost, D, E>)
    where
        D: EffectDialect,
    {
        cx.register(None, |child_cx| {
            child_cx.store_mut().push(Op::Add);
            let Self(lhs, rhs) = self;
            lhs.lower(child_cx);
            rhs.lower(child_cx);
        });
    }
}
