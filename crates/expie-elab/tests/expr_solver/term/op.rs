use expie_elab::{EffectDialect, Marker, MarkerRef, PassCx, Term};

use crate::{
    expr::{Num, Op},
    host::ExprHost,
};

/// Zero-argument operation term.
#[derive(Debug)]
pub struct Op0<'id>(Op<'id>);

/// [`Op`] [`Term`] impl for [`Op0`].
impl<'id, D, E> Term<'id, ExprHost, D, E> for Op0<'id> {
    fn lower(self, cx: &mut PassCx<'id, '_, '_, ExprHost, D, E>)
    where
        D: EffectDialect,
    {
        cx.register(None, |cx, _| {
            let Self(op) = self;
            cx.store_mut().push(op);
        });
    }
}

/// Pure value term.
/// We need a newtype wrapper ([`Op0`]) here for [`Term`] due to the orphan
/// rule.
#[must_use = "term constructor"]
pub const fn value<'id>(v: Num) -> Op0<'id> {
    Op0(Op::Value(v))
}

/// References a marked entity term via its [`MarkerRef`].
#[must_use = "term constructor"]
pub const fn var(marker: MarkerRef<'_>) -> Op0<'_> {
    Op0(Op::Var(marker))
}

/// One-argument operation term.
#[derive(Debug)]
pub struct Op1<'id, T> {
    term: T,
    op: Op<'id>,
    marker: Option<Marker<'id>>,
}

impl<'id, T> Op1<'id, T> {
    #[must_use = "term constructor"]
    pub const fn new(term: T, op: Op<'id>, marker: Option<Marker<'id>>) -> Self {
        Self { term, op, marker }
    }
}

/// [`Op`] [`Term`] impl for [`Op1`].
impl<'id, T, D, E> Term<'id, ExprHost, D, E> for Op1<'id, T>
where
    T: Term<'id, ExprHost, D, E>,
{
    fn lower(self, cx: &mut PassCx<'id, '_, '_, ExprHost, D, E>)
    where
        D: EffectDialect,
    {
        let Self { term, op, marker } = self;
        cx.register(marker, |cx, _| {
            cx.store_mut().push(op);
            term.lower(cx);
        });
    }
}

/// Negation operation term. Equivalent to:
/// ```
/// # let inner = 0.;
/// inner.neg();
/// ```
#[must_use = "term constructor"]
pub const fn neg<'id, T>(inner: T) -> Op1<'id, T> {
    Op1::new(inner, Op::Neg, None)
}

/// Tag an entity term with a unique [`Marker`].
///
/// [`Marker`]: expie_elab::Marker
#[must_use = "term constructor"]
pub const fn mark<T>(marker: Marker<'_>, inner: T) -> Op1<'_, T> {
    Op1::new(inner, Op::Identity, Some(marker))
}

/// N-ary product term that folds a sequence term.
#[must_use = "term constructor"]
pub const fn product<'id, T>(inner: T) -> Op1<'id, T> {
    Op1::new(inner, Op::Mul, None)
}

/// N-ary sum term that folds a sequence term.
#[must_use = "term constructor"]
pub const fn sum<'id, T>(inner: T) -> Op1<'id, T> {
    Op1::new(inner, Op::Add, None)
}

/// Two-argument operation term.
#[derive(Debug)]
pub struct Op2<'id, L, R> {
    lhs: L,
    rhs: R,
    op: Op<'id>,
}

impl<'id, L, R> Op2<'id, L, R> {
    #[must_use = "term constructor"]
    pub const fn new(lhs: L, rhs: R, op: Op<'id>) -> Self {
        Self { lhs, rhs, op }
    }
}

/// [`Op`] [`Term`] impl for [`Op2`].
impl<'id, L, R, D, E> Term<'id, ExprHost, D, E> for Op2<'id, L, R>
where
    L: Term<'id, ExprHost, D, E>,
    R: Term<'id, ExprHost, D, E>,
{
    fn lower(self, cx: &mut PassCx<'id, '_, '_, ExprHost, D, E>)
    where
        D: EffectDialect,
    {
        cx.register(None, |cx, _| {
            let Self { lhs, rhs, op } = self;
            cx.store_mut().push(op);
            lhs.lower(cx);
            rhs.lower(cx);
        });
    }
}

/// Addition operation term for two operands. Equivalent to:
/// ```
/// # let lhs = 0. let rhs = 0.;
/// lhs.add(rhs);
/// ```
#[must_use = "term constructor"]
pub const fn add<'id, L, R>(lhs: L, rhs: R) -> Op2<'id, L, R> {
    Op2::new(lhs, rhs, Op::Add)
}

/// Multiplication operation term for two operands. Equivalent to:
/// ```
/// # let lhs = 0. let rhs = 0.;
/// lhs.mul(rhs);
/// ```
#[must_use = "term constructor"]
pub const fn mul<'id, L, R>(lhs: L, rhs: R) -> Op2<'id, L, R> {
    Op2::new(lhs, rhs, Op::Mul)
}

/// Power operation term for two operands. Equivalent to:
/// ```
/// # let lhs = 0. let rhs = 0.;
/// lhs.pow(rhs);
/// ```
#[must_use = "term constructor"]
pub const fn pow<'id, L, R>(lhs: L, rhs: R) -> Op2<'id, L, R> {
    Op2::new(lhs, rhs, Op::Pow)
}
