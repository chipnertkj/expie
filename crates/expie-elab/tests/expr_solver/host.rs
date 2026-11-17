use expie_elab::{EffectDialect, Effects, Hierarchy, Host, Pass, PureDynTerm, trait_prelude::*};

use crate::{
    expr::{Expr, ExprStore, Num, PureExpr},
    solver::Solver,
};

/// Host for building and solving expressions.
#[derive(Debug, Clone, Copy)]
pub struct ExprHost;

impl ExprHost {
    /// Evaluate a [`Term`] expression constructed by `term_fn` and assert the
    /// lowered [`Expr`] matches the `expected` value within tolerence.
    ///
    /// [`Term`]: expie_elab::Term
    pub fn eval_assert<TFn>(expected: Num, term_fn: TFn)
    where
        TFn: for<'id> FnOnce(&mut Pass<'id, '_, Self>) -> Box<PureDynTerm<'id, Self>>,
    {
        Self::eval(term_fn, |mut o| o.assert_value(expected));
    }

    /// Evaluate a [`Term`] expression constructed by `term_fn` and apply the
    /// `output_fn` function to the lowered [`Expr`].
    ///
    /// [`Term`]: expie_elab::Term
    pub fn eval<TFn, OFn, O>(term_fn: TFn, output_fn: OFn) -> O
    where
        TFn: for<'id> FnOnce(&mut Pass<'id, '_, Self>) -> Box<PureDynTerm<'id, Self>>,
        OFn: for<'id> FnOnce(PureExpr<'id>) -> O,
    {
        Self.scope(|mut pass, mut effects| {
            let term = term_fn(&mut pass);
            pass.cx(&mut effects, |mut cx| term.lower_boxed(&mut cx));
            output_fn(pass.finalize(effects))
        })
    }
}

impl Host for ExprHost {
    type Store<'id> = ExprStore<'id>;
    type Output<'id, D, E>
        = Expr<'id, D, E>
    where
        D: EffectDialect + 'id,
        E: 'id;

    fn new_store<'id>(&self) -> Self::Store<'id> {
        ExprStore::default()
    }

    fn finalize_pass<'id, D, E>(
        &self,
        hierarchy: Hierarchy<'id>,
        store: Self::Store<'id>,
        effects: Effects<'id, D, E>,
    ) -> Self::Output<'id, D, E>
    where
        D: EffectDialect + 'id,
        E: 'id,
    {
        let ExprStore { ops, .. } = store;
        let values = Solver::new(&hierarchy, &ops).into_result().collect();
        Expr {
            hierarchy,
            ops,
            values,
            effects: effects.into_vec(),
        }
    }
}
