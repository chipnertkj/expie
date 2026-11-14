use expie_elab::{EffectDialect, ScopeCx, Term};

use crate::{expr::Op, host::ExprHost};

/// N-ary summation term that folds a sequence of terms into `Expr::Add`.
pub struct Sum<C>(pub C);

/// [`Term`] impl for [`Sum`].
impl<'id, C, D, E> Term<'id, ExprHost, D, E> for Sum<C>
where
    C: Term<'id, ExprHost, D, E>,
{
    fn lower(self, cx: &mut ScopeCx<'id, '_, ExprHost, D, E>)
    where
        D: EffectDialect,
    {
        cx.register(None, |child_cx| {
            child_cx.store_mut().push(Op::Add);
            let Self(term) = self;
            term.lower(child_cx);
        });
    }
}
