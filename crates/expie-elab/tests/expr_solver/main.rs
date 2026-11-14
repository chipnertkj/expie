//! Solver for primitive numerical expressions built using `expie-elab`.
//!
//! It implements a very simple arithmetic language compiler and evaluator.
//!
//! This example:
//! 1. Demonstrates `expie-elab`'s domain-agnosticism;
//! 2. Proves that [`Term`], [`Host`], [`Scope`], [`Marker`] etc. are truly
//!    general primitives for declarative tree elaboration;
//! 3. Shows the library's ability to express non-trivial relationships between
//!    entities (DAGs, variable binding), while keeping trivial hierarchies
//!    naturally expressible;
//! 4. Shows `Term`s flexibility in building sequence terms ([`Sum`],
//!    [`Product`]), conditional branches ([`Option`]), type erased nodes
//!    ([`ErasedTerm`]), and virtual terms ([`Mark`]);
//! 5. Exposes the library's intended two-phase separation between lowering
//!    (decode an expression) and evaluation (solve the expression). This is of
//!    course not enforced but highly favored by the design.
//!
//! Note that this example is intentionally kept minimal in terms of logic, and
//! as a result the solver:
//! - Indefinitely stalls on cycles (no cycle detection);
//! - Only showcases single-pass static evaluation (no reactivity or incremental
//!   evaluation).
//!
//! This example additionally exposes the ergonomic rough edges:
//! - Wrapper newtypes ([`Value`]) may be required to implement `Term` on
//!   existing types;
//! - Explicit type erasure ([`Box::new`] into `ErasedTerm` / [`TermErasure::erase`])
//!   is forced on every [`HostScope::scope`] call due to Rust's limited GAT support.
//!   See [RFC3216](https://github.com/rust-lang/rust/issues/97362) for more details.
//!
//! [`TermErasure::erase`]: expie_elab::TermErasure::erase

// TODO: make reactive now that we have effects!

use expie_elab::{TermIter, trait_prelude::*};

mod expr;
mod host;
mod solver;
mod term;

use self::{expr::Num, host::ExprHost, term::*};

/// `30 = (10 + 5) * 2`
#[test]
fn mul_add() {
    ExprHost::new().eval_assert_num(30., |_| Mul(Add(Value(10.), Value(5.)), Value(2.)).erase());
}

/// `9 = (-3) ^ 2`
#[test]
fn pow_neg_value() {
    ExprHost::new().eval_assert_num(9., |_| Pow(Neg(Value(3.)), Value(2.)).erase());
}

/// `10 = 1 + 2 + 3 + 4`
#[test]
fn sum_array() {
    ExprHost::new().eval_assert_num(10., |_| Sum([1., 2., 3., 4.].map(Value)).erase());
}

/// `24 = 2 * 3 * 4`
#[test]
fn product_vec() {
    ExprHost::new().eval_assert_num(24., |_| Product([2., 3., 4.].map(Value)).erase());
}

/// `15 = 10 + 5`
#[test]
fn option_some() {
    ExprHost::new().eval_assert_num(15., |_| Add(Value(10.), Some(Value(5.))).erase());
}

#[test]
fn option_none() {
    ExprHost::new().eval_assert_num(100., |_| Sum((Value(100.), None::<Value>)).erase());
}

/// `55 = 1 + 2 + 3 + 4 + 5 + 6 + 7 + 8 + 9 + 10`
#[test]
fn into_iter() {
    ExprHost::new().eval_assert_num(55., |_| {
        Sum(TermIter((1..=10).map(
            #[allow(clippy::cast_precision_loss, reason = "1..=10 < f32 mantissa max")]
            |i| Value(i as Num),
        )))
        .erase()
    });
}

/// `35 = { x = 5 } * 2 + x ^ 2`
#[test]
fn variable_binding() {
    ExprHost::new().eval_assert_num(35., |scope| {
        let (x, x_ref) = scope.marker();
        Add(
            Mul(Mark(x, Value(5.)), Value(2.)),
            Pow(Var(x_ref), Value(2.)),
        )
        .erase()
    });
}

/// `30 = { x = 10 } + x + x`
#[test]
fn reusing_marked_subexpression() {
    ExprHost::new().eval_assert_num(30., |scope| {
        let (x, x_ref) = scope.marker();
        Sum((Mark(x, Value(10.)), Var(x_ref), Var(x_ref))).erase()
    });
}

/// `30 = (10 + 5) * 2`
///
/// Instead of just evaluating, also inspects and verifies hierarchy.
#[test]
fn test_hierarchy_structure() {
    let host = ExprHost::new();

    // Expression: (10. + 5.) * 2.
    host.eval(
        |_| Mul(Add(Value(10.), Value(5.)), Value(2.)).erase(),
        |expr| {
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
