use expie_elab::{EffectDialect, ScopeCx, Term};

use crate::{expr::Op, host::ExprHost};

/// N-ary product term that folds a sequence of terms into `Expr::Mul`.
pub struct Product<C>(pub C);

/// [`Term`] impl for [`Product`].
impl<'id, C, D, E> Term<'id, ExprHost, D, E> for Product<C>
where
    C: Term<'id, ExprHost, D, E>,
{
    fn lower(self, cx: &mut ScopeCx<'id, '_, ExprHost, D, E>)
    where
        D: EffectDialect,
    {
        cx.register(None, |child_cx| {
            child_cx.store_mut().push(Op::Mul);
            let Self(term) = self;
            term.lower(child_cx);
        });
    }
}
