use expie_elab::{EffectDialect, ScopeCx, Term};

use crate::{expr::Op, host::ExprHost};

/// Negation term.
pub struct Neg<T>(pub T);

/// [`Term`] impl for [`Neg`].
impl<'id, T, D, E> Term<'id, ExprHost, D, E> for Neg<T>
where
    T: Term<'id, ExprHost, D, E>,
{
    fn lower(self, cx: &mut ScopeCx<'id, '_, ExprHost, D, E>)
    where
        D: EffectDialect,
    {
        cx.register(None, |child_cx| {
            child_cx.store_mut().push(Op::Neg);
            let Self(term) = self;
            term.lower(child_cx);
        });
    }
}
