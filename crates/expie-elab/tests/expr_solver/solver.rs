use expie_elab::{Hierarchy, MarkerRef, NodeId};

use crate::expr::{Num, Op};

/// Solver context.
pub struct Solver<'output, 'id> {
    hierarchy: &'output Hierarchy<'id>,
    /// [`NodeId`] indexed lowered operations.
    ops: &'output [Op<'id>],
    /// [`NodeId`] indexed computed values.
    values: Vec<Option<Num>>,
}

impl<'output, 'id> Solver<'output, 'id> {
    /// Create a new solver context based on the lowered expression.
    pub fn new(hierarchy: &'output Hierarchy<'id>, ops: &'output [Op<'id>]) -> Self {
        Self {
            hierarchy,
            ops,
            values: vec![None; hierarchy.nodes_len()],
        }
    }

    /// Solve the expression and return the computed values for each node.
    pub fn into_result(mut self) -> Vec<Num> {
        let root = self
            .hierarchy
            .main_root()
            .expect("empty expression hierarchy");
        self.solve(root);
        self.values
            .into_iter()
            .map(|v| v.expect("unresolved node"))
            .collect()
    }

    /// Get the operation for a specific node.
    fn op(&self, node: NodeId<'id>) -> Op<'id> {
        let ix = node.erased().raw_value();
        self.ops
            .get(ix)
            .copied()
            .expect("unresolved node operation")
    }

    /// Get the value for a specific node.
    fn value(&self, node: NodeId<'id>) -> Option<Num> {
        let ix = node.erased().raw_value();
        self.values.get(ix).copied().expect("unresolved node value")
    }

    /// Get a mutable reference to the value for a specific node.
    fn value_mut(&mut self, node: NodeId<'id>) -> &mut Option<Num> {
        let ix = node.erased().raw_value();
        self.values.get_mut(ix).expect("unresolved node value")
    }

    /// Solve the value for [`Op::Neg`].
    fn solve_neg(&mut self, mut children: impl Iterator<Item = NodeId<'id>>) -> Num {
        let child = children.next().expect("missing operand for Neg");
        self.solve(child);
        -self.value(child).expect("just computed neg child")
    }

    /// Solve the value for [`Op::Add`].
    fn solve_add(&mut self, children: impl Iterator<Item = NodeId<'id>>) -> Num {
        children
            .map(|c| {
                self.solve(c);
                self.value(c).expect("just computed add child")
            })
            .sum()
    }

    /// Solve the value for [`Op::Mul`].
    fn solve_mul(&mut self, children: impl Iterator<Item = NodeId<'id>>) -> Num {
        children
            .map(|c| {
                self.solve(c);
                self.value(c).expect("just computed mul child")
            })
            .product()
    }

    /// Solve the value for [`Op::Pow`].
    fn solve_pow(&mut self, mut children: impl Iterator<Item = NodeId<'id>>) -> Num {
        let base_node = children.next().expect("missing base for Pow");
        let exp_node = children.next().expect("missing exponent for Pow");
        self.solve(base_node);
        self.solve(exp_node);
        let base = self.value(base_node).expect("just computed base");
        let exp = self.value(exp_node).expect("just computed exponent");
        base.powf(exp)
    }

    /// Solve the value for [`Op::Var`].
    fn solve_var(&mut self, marker_ref: MarkerRef<'id>) -> Num {
        let target_node = self
            .hierarchy
            .marked_node(marker_ref)
            .expect("unresolved marker reference");
        self.solve(target_node);
        self.value(target_node).expect("just computed var")
    }

    /// Solve the value for [`Op::Identity`].
    fn solve_identity(&mut self, mut children: impl Iterator<Item = NodeId<'id>>) -> Num {
        let child = children.next().expect("missing child for Identity");
        self.solve(child);
        self.value(child).expect("just computed identity child")
    }

    /// Solve the value of a specific node in the hierarchy.
    fn solve(&mut self, node: NodeId<'id>) {
        // If the value is already computed, early return.
        if self.value(node).is_some() {
            return;
        }

        // Otherwise, compute the value.
        let op = self.op(node);
        let children = self.hierarchy.children_of(node);
        let result = match op {
            // Pure value nodes are solver seeders.
            Op::Value(val) => val,
            // Negates the computed value of the child node.
            Op::Neg => self.solve_neg(children),
            // Sums the computed values of the child nodes.
            Op::Add => self.solve_add(children),
            // Multiplies the computed values of the child nodes.
            Op::Mul => self.solve_mul(children),
            // Raises the computed value of the base node to the power of the computed value of
            // the exponent node.
            Op::Pow => self.solve_pow(children),
            // References a marked entity term via its [`MarkerRef`].
            Op::Var(marker_ref) => self.solve_var(marker_ref),
            // Used to tag a marked entity term with a unique [`Marker`].
            Op::Identity => self.solve_identity(children),
        };

        // Store the computed value for the node.
        *self.value_mut(node) = Some(result);
    }
}
