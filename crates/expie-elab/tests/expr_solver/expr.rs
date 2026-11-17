use std::convert::Infallible;

use expie_elab::{EffectDialect, Hierarchy, MarkerRef, NodeId};

use crate::{
    reactive::{Reactive, ReactiveBinding, ReactiveEffect},
    solver::Solver,
};

/// Number type used for arithmetic operations.
pub type Num = f32;

/// Assert that a number is close to the expected value within a small
/// epsilon (floating point comparison tolerance).
fn assert_num(v: Num, expected: Num) {
    const EPSILON: Num = 1e-6;
    assert!((v - expected).abs() < EPSILON);
}

/// Operation kind for a node in the expression hierarchy.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Op<'id> {
    /// Pure fixed value, seeds the solver.
    Value(Num),
    /// Negates the value of the child node.
    Neg,
    /// Sums the values two child nodes.
    Add,
    /// Multiplies the values of two child nodes.
    Mul,
    /// Raises the value of the base node to the power of the value of the
    /// exponent node.
    Pow,
    /// Variable reference node.
    /// References a marked entity term via its [`MarkerRef`].
    Var(MarkerRef<'id>),
    /// Identity node.
    /// Used to tag a marked entity term with a unique [`Marker`].
    ///
    /// [`Marker`]: expie_elab::Marker
    Identity,
}

#[derive(Debug, Default)]
pub struct ExprStore<'id> {
    pub(crate) ops: Vec<Op<'id>>,
}

impl<'id> ExprStore<'id> {
    pub fn push(&mut self, op: Op<'id>) {
        self.ops.push(op);
    }
}

/// Lowered expression represented via a [`Hierarchy`] of nodes and computed
/// values.
#[derive(Debug)]
pub struct Expr<'id, D: EffectDialect, E> {
    pub hierarchy: Hierarchy<'id>,
    pub ops: Vec<Op<'id>>,
    pub values: Vec<Option<Num>>,
    pub effects: Vec<D::Binding<'id, E>>,
}

/// An [`Expr`] that has no effects and isn't compatible with an effect dialect
/// (`D = Infallible`, `E = Infallible`).
pub type PureExpr<'id> = Expr<'id, Infallible, Infallible>;

impl<'id, D: EffectDialect, E> Expr<'id, D, E> {
    /// Evaluate the result value of the root expression.
    pub fn value(&mut self) -> Num {
        self.ensure_solved();
        self.hierarchy
            .main_root()
            .map(|node| self.values[node.erased().raw_value()].expect("root not solved"))
            .unwrap_or_default() // no root => 0.
    }

    /// Get the computed value of a specific node in the hierarchy.
    pub fn value_of(&mut self, node: NodeId<'id>) -> Num {
        self.ensure_solved();
        self.values[node.erased().raw_value()].expect("node not solved")
    }

    /// Assert that the root value of the expression is equal to the expected
    /// value, within tolerence.
    pub fn assert_value(&mut self, expected: Num) {
        assert_num(self.value(), expected);
    }

    /// Assert that the value of a specific node in the hierarchy is equal to
    /// the expected value, within tolerence.
    pub fn assert_value_of(&mut self, node: NodeId<'id>, expected: Num) {
        assert_num(self.value_of(node), expected);
    }

    fn ensure_solved(&mut self) {
        if self.values.len() == self.hierarchy.nodes_len()
            && self.values.iter().all(Option::is_some)
        {
            return;
        }
        self.values = Solver::new(&self.hierarchy, &self.ops)
            .into_result()
            .collect();
    }
}

impl<'id> Expr<'id, Reactive, ReactiveEffect> {
    /// Writable input nodes registered via [`Track`] effects.
    ///
    /// [`Track`]: crate::term::track
    pub fn tracked_inputs(&self) -> impl Iterator<Item = NodeId<'id>> + '_ {
        self.effects.iter().map(|binding| {
            let ReactiveBinding::ValueChange { node, .. } = *binding;
            node
        })
    }

    /// Mutate a tracked [`Op::Value`] associated with `marker`.
    pub fn set_value(&mut self, marker: MarkerRef<'id>, value: Num) {
        let id = self.tracked_for_marker(marker);
        match self.ops.get_mut(id.erased().raw_value()) {
            Some(Op::Value(slot)) => *slot = value,
            Some(other) => panic!("expected Op::Value at tracked {id}, got {other:?}"),
            None => panic!("missing op at tracked {id}"),
        }
        self.invalidate_all();
    }

    /// Resolve `marker` to a tracked input [`NodeId`] from [`Self::effects`].
    fn tracked_for_marker(&self, marker: MarkerRef<'id>) -> NodeId<'id> {
        let anchor = self
            .hierarchy
            .marked_node(marker)
            .expect("marked node not found");
        self.tracked_inputs()
            .find(|&tracked| self.tracked_belongs_to_mark(anchor, tracked))
            .expect("marker is not associated with a tracked input")
    }

    /// True when `tracked` is `anchor`, or nested under it (identity wrapper).
    fn tracked_belongs_to_mark(&self, anchor: NodeId<'id>, tracked: NodeId<'id>) -> bool {
        let mut walk = Some(tracked);
        while let Some(node) = walk {
            if node == anchor {
                return true;
            }
            walk = self.hierarchy.parent_of(node);
        }
        false
    }

    /// Drop the whole value cache (coarse reactivity).
    fn invalidate_all(&mut self) {
        self.values.iter_mut().for_each(|v| *v = None);
    }
}
