use expie_elab::{EffectDialect, ScopeCx, Term};

use crate::{expr::Op, host::ExprHost};

/// Power term.
pub struct Pow<L, R>(pub L, pub R);

/// [`Term`] impl for [`Pow`].
impl<'id, L, R, D, E> Term<'id, ExprHost, D, E> for Pow<L, R>
where
    L: Term<'id, ExprHost, D, E>,
    R: Term<'id, ExprHost, D, E>,
{
    fn lower(self, cx: &mut ScopeCx<'id, '_, ExprHost, D, E>)
    where
        D: EffectDialect,
    {
        cx.register(None, |child_cx| {
            child_cx.store_mut().push(Op::Pow);
            let Self(lhs, rhs) = self;
            lhs.lower(child_cx);
            rhs.lower(child_cx);
        });
    }
}
