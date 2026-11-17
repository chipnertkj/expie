//! Solver for primitive numerical expressions built using `expie-elab`.
//!
//! It implements a very simple arithmetic language compiler and evaluator.
//!
//! This example:
//! 1. Demonstrates `expie-elab`'s domain-agnosticism;
//! 2. Shows that [`Term`], [`Host`], [`Pass`], [`Marker`] etc. work as general
//!    primitives for declarative tree elaboration;
//! 3. Shows the library's ability to express non-trivial relationships between
//!    entities (DAGs, variable binding), while keeping trivial hierarchies
//!    naturally expressible;
//! 4. Shows `Term`s flexibility in building sequence terms ([`sum`],
//!    [`product`]), conditional branches ([`Option`]), type erased nodes
//!    ([`ErasedTerm`]), and virtual terms ([`mark`]);
//! 5. Exposes the library's intended two-phase separation between lowering
//!    (decode an expression) and evaluation (solve the expression). This is of
//!    course not enforced but highly favored by the design.
//! 6. Registers tracked inputs through [`Effects`]. This could instead be done
//!    through [`Store`] lowering, but `Effects` is parameterized by the
//!    [`EffectDialect`] in use, which separates generic arithmetic terms from
//!    those depending on [`Reactive`] and makes reactivity opt-in.
//!
//! [`Term`]: expie_elab::Term
//! [`Host`]: expie_elab::Host
//! [`Pass`]: expie_elab::Pass
//! [`Marker`]: expie_elab::Marker
//! [`ErasedTerm`]: expie_elab::ErasedTerm
//! [`Effects`]: expie_elab::Effects
//! [`Store`]: expie_elab::Host::Store
//! [`EffectDialect`]: expie_elab::EffectDialect
//! [`Reactive`]: crate::reactive::Reactive
//!
//! Note that this example is intentionally kept minimal in terms of logic, and
//! as a result:
//! - The solver indefinitely stalls on cycles;
//! - Reactivity is implemented only via coarse invalidation.
//!
//! This example additionally exposes the ergonomic rough edges:
//! - Wrapper newtypes ([`value`]) may be required to implement `Term` on
//!   existing types;
//! - Explicit type erasure ([`TermErasure::erase`]) is forced on every closure
//!   returning `impl Term` due to Rust's limited GAT support.
//!   See [RFC3216](https://github.com/rust-lang/rust/issues/97362) for more details.
//!   This isn't necessary for regular scope usage, but it is required for returning
//!   a term from a closure. These closures additionally have to be inline.
//!
//! [`TermErasure::erase`]: expie_elab::TermErasure::erase

use expie_elab::{TermIter, trait_prelude::*};

mod expr;
mod host;
mod reactive;
mod solver;
pub mod term;

use self::{expr::Num, host::ExprHost, term::*};

/// `30 = (10 + 5) * 2`
#[test]
fn mul_add() {
    ExprHost::eval_assert(30., |_| mul(add(value(10.), value(5.)), value(2.)).erase());
}

/// `9 = (-3) ^ 2`
#[test]
fn pow_neg_value() {
    ExprHost::eval_assert(9., |_| pow(neg(value(3.)), value(2.)).erase());
}

/// `10 = 1 + 2 + 3 + 4`
#[test]
fn sum_array() {
    ExprHost::eval_assert(10., |_| sum([1., 2., 3., 4.].map(value)).erase());
}

/// `24 = 2 * 3 * 4`
#[test]
fn product_vec() {
    ExprHost::eval_assert(24., |_| product([2., 3., 4.].map(value)).erase());
}

/// `15 = 10 + 5`
#[test]
fn option_some() {
    ExprHost::eval_assert(15., |_| add(value(10.), Some(value(5.))).erase());
}

#[test]
fn option_none() {
    ExprHost::eval_assert(100., |_| sum((value(100.), None::<Op0>)).erase());
}

/// `55 = (1..=10).sum()`
#[test]
fn into_iter() {
    ExprHost::eval_assert(55., |_| {
        sum(TermIter((1..=10).map(
            #[allow(clippy::cast_precision_loss, reason = "1..=10 < f32 mantissa max")]
            |i| value(i as Num),
        )))
        .erase()
    });
}

/// 1. `let x;`
/// 2. `35 = { x = 5; x } * 2 + x ^ 2`
#[test]
fn variable_binding() {
    ExprHost::eval_assert(35., |scope| {
        let (x, x_ref) = scope.marker();
        add(
            mul(mark(x, value(5.)), value(2.)),
            pow(var(x_ref), value(2.)),
        )
        .erase()
    });
}

/// 1. `let x;`
/// 2. `30 = { x = 10; x } + x + x`
#[test]
fn reusing_marked_subexpression() {
    ExprHost::eval_assert(30., |scope| {
        let (x, x_ref) = scope.marker();
        sum((mark(x, value(10.)), var(x_ref), var(x_ref))).erase()
    });
}

/// Track input `x` reactively, mutate, recompute expression value.
///
/// 1. `let x;`
/// 2. `35 = { x = 5; x } * 2 + x ^ 2`
/// 3. `x := 10`
/// 4. `120 = x * 2 + x ^ 2`
#[test]
fn reactive_tracked_input() {
    ExprHost.scope(|mut pass, mut effects| {
        let (x, x_ref) = pass.marker();
        let term = add(
            mul(mark(x, track(5.)), value(2.)),
            pow(var(x_ref), value(2.)),
        )
        .erase();
        pass.cx(&mut effects, |mut cx| term.lower_boxed(&mut cx));
        let mut expr = pass.finalize(effects);
        expr.assert_value(35.);
        expr.set_value(x_ref, 10.);
        expr.assert_value(120.);
    });
}

/// Instead of just evaluating, also inspects and verifies hierarchy.
///
/// `30 = (10 + 5) * 2`
#[test]
fn test_hierarchy_structure() {
    // Expression: (10. + 5.) * 2.
    ExprHost::eval(
        |_| mul(add(value(10.), value(5.)), value(2.)).erase(),
        |mut expr| {
            // Root is Node 0 (Mul), total evaluated value is 30.0
            expr.assert_value(30.0);

            // Mul has two children: Add and Value(2.)
            let root = expr
                .hierarchy
                .main_root()
                .expect("empty expression hierarchy");
            let children: Vec<_> = expr.hierarchy.children_of(root).collect();
            assert_eq!(children.len(), 2);
            expr.assert_value_of(children[0], 15.0); // Add (10 + 5)
            expr.assert_value_of(children[1], 2.0); // Value(2)

            // Add has two children: Value(10.) and Value(5.)
            let add_children: Vec<_> = expr.hierarchy.children_of(children[0]).collect();
            assert_eq!(add_children.len(), 2);
            expr.assert_value_of(add_children[0], 10.0);
            expr.assert_value_of(add_children[1], 5.0);

            // Total nodes in the hierarchy: Mul + Add + Value(10.) + Value(5.)
            // + Value(2.) = 5
            assert_eq!(expr.hierarchy.nodes_len(), 5);
        },
    );
}
