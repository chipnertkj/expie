use expie_elab::{EffectDialect, Marker, ScopeCx, Term};

use crate::{expr::Op, host::ExprHost};

/// Tag an entity term with a unique [`Marker`].
pub struct Mark<'id, T>(pub Marker<'id>, pub T);

/// [`Term`] impl for [`Mark`].
impl<'id, T, D, E> Term<'id, ExprHost, D, E> for Mark<'id, T>
where
    T: Term<'id, ExprHost, D, E>,
{
    fn lower(self, cx: &mut ScopeCx<'id, '_, ExprHost, D, E>)
    where
        D: EffectDialect,
    {
        let Self(marker, term) = self;
        cx.register(Some(marker), |child_cx| {
            child_cx.store_mut().push(Op::Identity);
            term.lower(child_cx);
        });
    }
}
