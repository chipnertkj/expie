use expie_elab::{
    EffectDialect, Effects, Hierarchy, Host, PureTermFnOnce, ScopeOutputFnOnce, trait_prelude::*,
};

use crate::{
    expr::{Expr, Num, Op},
    solver::Solver,
};

/// Host for building and solving expressions.
pub struct ExprHost;

impl ExprHost {
    pub const fn new() -> Self {
        Self
    }

    /// Evaluate a term expression and assert the lowered expression matches the
    /// expected value.
    pub fn eval_assert_num<TFn>(&self, expected: Num, f: TFn)
    where
        TFn: for<'id> PureTermFnOnce<'id, Self>,
    {
        self.scope(f, |o| o.assert_value(expected));
    }

    /// Evaluate a term expression and apply an output function to the lowered
    /// expression.
    pub fn eval<TFn, OFn, O>(&self, term_fn: TFn, output_fn: OFn) -> O
    where
        TFn: for<'id> PureTermFnOnce<'id, Self>,
        OFn: for<'id> ScopeOutputFnOnce<'id, Self, O>,
    {
        self.scope(term_fn, output_fn)
    }
}

impl Host for ExprHost {
    type Store<'id> = Vec<Op<'id>>;
    type Output<'id> = Expr<'id>;

    fn new_store<'id>(&self) -> Self::Store<'id> {
        Vec::new()
    }

    fn finalize_scope<'id, D: EffectDialect, E>(
        &self,
        hierarchy: Hierarchy<'id>,
        ops: Self::Store<'id>,
        _effects: Effects<D, E>,
    ) -> Self::Output<'id> {
        let values = Solver::new(&hierarchy, &ops).into_result();
        Expr { hierarchy, values }
    }
}
