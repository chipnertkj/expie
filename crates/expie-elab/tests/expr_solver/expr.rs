use expie_elab::{Hierarchy, MarkerRef, NodeId};

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

/// Lowered expression represented via a [`Hierarchy`] of nodes and computed
/// values.
#[derive(Debug)]
pub struct Expr<'id> {
    pub hierarchy: Hierarchy<'id>,
    pub values: Vec<Num>,
}

impl<'id> Expr<'id> {
    /// Evaluate the result value of the root expression.
    pub fn value(&self) -> Num {
        self.hierarchy
            .main_root()
            .map(|node| self.value_of(node))
            .unwrap_or_default()
    }

    /// Get the computed value of a specific node in the hierarchy.
    pub fn value_of(&self, node: NodeId<'id>) -> Num {
        self.values[node.erased().raw_value()]
    }

    /// Assert that the root value of the expression is equal to the expected
    /// value, within tolerence.
    pub fn assert_value(&self, expected: Num) {
        assert_num(self.value(), expected);
    }

    /// Assert that the value of a specific node in the hierarchy is equal to
    /// the expected value, within tolerence.
    pub fn assert_value_of(&self, node: NodeId<'id>, expected: Num) {
        assert_num(self.value_of(node), expected);
    }
}
