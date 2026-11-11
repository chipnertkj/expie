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
//! - Explicit type erasure ([`Box::new`] into `ErasedTerm`) is forced on every
//!   [`HostScope::scope`] call due to Rust's limited GAT support. See
//!   [RFC3216](https://github.com/rust-lang/rust/issues/97362) for more details.

use expie_elab::{
    ErasedTerm, Hierarchy, Host, HostScope, Marker, MarkerRef, NodeId, Scope, ScopeCx, Term,
    TermIter,
};

#[test]
fn test_expression_lowering_and_eval() {
    let host = ExprHost::new();

    // Expression: (10. + 5.) * 2. = 30.
    host.eval_assert(30., |_| {
        Box::new(Mul(Add(Value(10.), Value(5.)), Value(2.)))
    });

    // Expression: (-3.) ^ 2. = 9.
    host.eval_assert(9., |_| Box::new(Pow(Neg(Value(3.)), Value(2.))));

    // Array sequence term [T; N]: 1. + 2. + 3. + 4. = 10.
    host.eval_assert(10., |_| Box::new(Sum([1., 2., 3., 4.].map(Value))));

    // Direct Vec sequence term: 2. * 3. * 4. = 24.
    host.eval_assert(24., |_| {
        Box::new(Product(
            // Vec of Term is a sequence Term.
            [2., 3., 4.].into_iter().map(Value).collect::<Vec<_>>(),
        ))
    });

    // Direct Option term (when Some): 10. + 5. = 15.
    host.eval_assert(15., |_| {
        Box::new(Add(
            Value(10.),
            // Option of Term is a conditional Term.
            Some(Value(5.)),
        ))
    });

    // Optional sequence terms: 100. + Some(20.) + None = 120.
    host.eval_assert(120., |_| {
        Box::new(Sum((Value(100.), Some(Value(20.)), None::<Value>)))
    });

    // IntoIterator: 1 + 2 + ... + 10 = 55.
    host.eval_assert(55., |_| {
        Box::new(Sum(
            // Use TermIter as an adapter for arbitrary iterators.
            TermIter((1..=10).map(
                #[allow(clippy::cast_precision_loss, reason = "1..=10 < f32 mantissa max")]
                |i| Value(i as Num),
            )),
        ))
    });
}

/// Expression with variable binding:
/// ```
/// let x;
/// assert_eq!(({ x = 5; x } * 2.) + (x ^ 2.), 35.);
/// ```
#[test]
fn variable_binding() {
    ExprHost::new().eval_assert(35., |scope| {
        let (x, x_ref) = scope.marker();
        Box::new(Add(
            Mul(Mark(x, Value(5.)), Value(2.)),
            Pow(Var(x_ref), Value(2.)),
        ))
    });
}

/// Reusing a marked sub-expression multiple times in a sum:
/// ```
/// let x;
/// assert_eq!(({ x = 10; x } + x) + x, 30.);
/// ```
#[test]
fn reusing_marked_subexpression() {
    ExprHost::new().eval_assert(30., |scope| {
        let (x, x_ref) = scope.marker();
        Box::new(Sum((Mark(x, Value(10.)), Var(x_ref), Var(x_ref))))
    });
}

#[test]
fn test_hierarchy_structure() {
    let host = ExprHost::new();

    // Expression: (10. + 5.) * 2.
    host.scope(
        |_| Box::new(Mul(Add(Value(10.), Value(5.)), Value(2.))),
        |expr| {
            // Root is Node 0 (Mul), total evaluated value is 30.0
            assert_num(expr.value(), 30.0);

            // Mul has two children: Add and Value(2.)
            let root = expr
                .hierarchy
                .main_root()
                .expect("empty expression hierarchy");
            let children: Vec<_> = expr.hierarchy.children_of(root).collect();
            assert_eq!(children.len(), 2);
            assert_num(expr.value_of(children[0]), 15.0); // Add (10 + 5)
            assert_num(expr.value_of(children[1]), 2.0); // Value(2)

            // Add has two children: Value(10.) and Value(5.)
            let add_children: Vec<_> = expr.hierarchy.children_of(children[0]).collect();
            assert_eq!(add_children.len(), 2);
            assert_num(expr.value_of(add_children[0]), 10.0);
            assert_num(expr.value_of(add_children[1]), 5.0);

            // Total nodes in the hierarchy: Mul + Add + Value(10.) + Value(5.)
            // + Value(2.) = 5
            assert_eq!(expr.hierarchy.nodes_len(), 5);
        },
    );
}

/// Number type used for arithmetic operations.
type Num = f32;

/// Host for building and solving expressions.
struct ExprHost;

impl ExprHost {
    pub const fn new() -> Self {
        Self
    }
}

impl Host for ExprHost {
    type Store<'id> = Vec<Op<'id>>;
    type Output<'id> = Expr<'id>;

    fn new_store<'id>(&self) -> Self::Store<'id> {
        Vec::new()
    }

    fn finalize_scope<'id>(
        &self,
        hierarchy: Hierarchy<'id>,
        ops: Self::Store<'id>,
    ) -> Self::Output<'id> {
        let values = Evaluator::new(&hierarchy, &ops).into_result();
        Expr { hierarchy, values }
    }
}

/// Operation kind for a node in the expression hierarchy.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Op<'id> {
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
    Identity,
}

/// Lowered expression represented via a [`Hierarchy`] of nodes and computed
/// values.
#[derive(Debug)]
struct Expr<'id> {
    hierarchy: Hierarchy<'id>,
    values: Vec<Num>,
}

impl<'id> Expr<'id> {
    /// Evaluate the result value of the root expression.
    fn value(&self) -> Num {
        self.hierarchy
            .main_root()
            .map(|node| self.value_of(node))
            .unwrap_or_default()
    }

    /// Get the computed value of a specific node in the hierarchy.
    fn value_of(&self, node: NodeId<'id>) -> Num {
        self.values[node.erased().raw_value()]
    }
}

/// Solver context.
struct Evaluator<'output, 'id> {
    hierarchy: &'output Hierarchy<'id>,
    ops: &'output [Op<'id>],
    values: Vec<Option<Num>>,
}

impl<'output, 'id> Evaluator<'output, 'id> {
    /// Create a new solver context based on the lowered expression.
    fn new(hierarchy: &'output Hierarchy<'id>, ops: &'output [Op<'id>]) -> Self {
        Self {
            hierarchy,
            ops,
            values: vec![None; hierarchy.nodes_len()],
        }
    }

    /// Solve the expression and return the computed values for each node.
    fn into_result(mut self) -> Vec<Num> {
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

    /// Solve the value of a specific node in the hierarchy.
    fn solve(&mut self, node: NodeId<'id>) {
        let ix = node.erased().raw_value();
        // If the value is already computed, early return.
        if self.values[ix].is_some() {
            return;
        }

        // Otherwise, compute the value.
        let op = self.ops[ix];
        let mut children = self.hierarchy.children_of(node);
        let result = match op {
            // Pure value nodes are solver seeders.
            Op::Value(val) => val,
            // Negates the computed value of the child node.
            Op::Neg => {
                let child = children.next().expect("missing operand for Neg");
                self.solve(child);
                let ix = child.erased().raw_value();
                -self.values[ix].expect("just computed neg child")
            }
            // Sums the computed values of the child nodes.
            Op::Add => children
                .map(|c| {
                    self.solve(c);
                    let ix = c.erased().raw_value();
                    self.values[ix].expect("just computed add child")
                })
                .sum(),
            // Multiplies the computed values of the child nodes.
            Op::Mul => children
                .map(|c| {
                    self.solve(c);
                    let ix = c.erased().raw_value();
                    self.values[ix].expect("just computed mul child")
                })
                .product(),
            // Raises the computed value of the base node to the power of the computed value of the
            // exponent node.
            Op::Pow => {
                let base_node = children.next().expect("missing base for Pow");
                let exp_node = children.next().expect("missing exponent for Pow");
                self.solve(base_node);
                self.solve(exp_node);
                let base_ix = base_node.erased().raw_value();
                let exp_ix = exp_node.erased().raw_value();
                let base = self.values[base_ix].expect("just computed base");
                let exp = self.values[exp_ix].expect("just computed exponent");
                base.powf(exp)
            }
            // References a marked entity term via its [`MarkerRef`].
            Op::Var(marker_ref) => {
                let target_node = self
                    .hierarchy
                    .marked_node(marker_ref)
                    .expect("unresolved marker reference");
                self.solve(target_node);
                let ix = target_node.erased().raw_value();
                self.values[ix].expect("just computed var")
            }
            // Used to tag a marked entity term with a unique [`Marker`].
            Op::Identity => {
                let child = children.next().expect("missing child for Identity");
                self.solve(child);
                let ix = child.erased().raw_value();
                self.values[ix].expect("just computed identity child")
            }
        };

        // Store the computed value for the node.
        self.values[ix] = Some(result);
    }
}

/// Pure value term.
/// We need a newtype wrapper here for [`Term`] due to the orphan rule.
struct Value(Num);
/// Negation term.
struct Neg<T>(T);
/// Addition term.
struct Add<L, R>(L, R);
/// Multiplication term.
struct Mul<L, R>(L, R);
/// Power term.
struct Pow<L, R>(L, R);
/// N-ary summation term that folds a sequence of terms into `Expr::Add`.
struct Sum<C>(C);
/// N-ary product term that folds a sequence of terms into `Expr::Mul`.
struct Product<C>(C);
/// Tag an entity term with a unique [`Marker`].
struct Mark<'id, T>(Marker<'id>, T);
/// Reference a marked entity term via its [`MarkerRef`].
struct Var<'id>(MarkerRef<'id>);

/// [`Term`] impl for [`Value`].
impl<'id> Term<'id, ExprHost, ()> for Value {
    fn lower(self, cx: &mut ScopeCx<'id, '_, ExprHost>) {
        cx.register(None, |child_cx| {
            child_cx.store_mut().push(Op::Value(self.0));
        });
    }
}

/// [`Term`] impl for [`Neg`].
impl<'id, T> Term<'id, ExprHost, ()> for Neg<T>
where
    T: Term<'id, ExprHost, ()>,
{
    fn lower(self, cx: &mut ScopeCx<'id, '_, ExprHost>) {
        cx.register(None, |child_cx| {
            child_cx.store_mut().push(Op::Neg);
            self.0.lower(child_cx);
        });
    }
}

/// [`Term`] impl for [`Add`].
impl<'id, L, R> Term<'id, ExprHost, ()> for Add<L, R>
where
    L: Term<'id, ExprHost, ()>,
    R: Term<'id, ExprHost, ()>,
{
    fn lower(self, cx: &mut ScopeCx<'id, '_, ExprHost>) {
        cx.register(None, |child_cx| {
            child_cx.store_mut().push(Op::Add);
            self.0.lower(child_cx);
            self.1.lower(child_cx);
        });
    }
}

/// [`Term`] impl for [`Mul`].
impl<'id, L, R> Term<'id, ExprHost, ()> for Mul<L, R>
where
    L: Term<'id, ExprHost, ()>,
    R: Term<'id, ExprHost, ()>,
{
    fn lower(self, cx: &mut ScopeCx<'id, '_, ExprHost>) {
        cx.register(None, |child_cx| {
            child_cx.store_mut().push(Op::Mul);
            self.0.lower(child_cx);
            self.1.lower(child_cx);
        });
    }
}

/// [`Term`] impl for [`Pow`].
impl<'id, L, R> Term<'id, ExprHost, ()> for Pow<L, R>
where
    L: Term<'id, ExprHost, ()>,
    R: Term<'id, ExprHost, ()>,
{
    fn lower(self, cx: &mut ScopeCx<'id, '_, ExprHost>) {
        cx.register(None, |child_cx| {
            child_cx.store_mut().push(Op::Pow);
            self.0.lower(child_cx);
            self.1.lower(child_cx);
        });
    }
}

/// [`Term`] impl for [`Sum`].
impl<'id, C> Term<'id, ExprHost, ()> for Sum<C>
where
    C: Term<'id, ExprHost, ()>,
{
    fn lower(self, cx: &mut ScopeCx<'id, '_, ExprHost>) {
        cx.register(None, |child_cx| {
            child_cx.store_mut().push(Op::Add);
            self.0.lower(child_cx);
        });
    }
}

/// [`Term`] impl for [`Product`].
impl<'id, C> Term<'id, ExprHost, ()> for Product<C>
where
    C: Term<'id, ExprHost, ()>,
{
    fn lower(self, cx: &mut ScopeCx<'id, '_, ExprHost>) {
        cx.register(None, |child_cx| {
            child_cx.store_mut().push(Op::Mul);
            self.0.lower(child_cx);
        });
    }
}

/// [`Term`] impl for [`Mark`].
impl<'id, T> Term<'id, ExprHost, ()> for Mark<'id, T>
where
    T: Term<'id, ExprHost, ()>,
{
    fn lower(self, cx: &mut ScopeCx<'id, '_, ExprHost>) {
        cx.register(Some(self.0), |child_cx| {
            child_cx.store_mut().push(Op::Identity);
            self.1.lower(child_cx);
        });
    }
}

/// [`Term`] impl for [`Var`].
impl<'id> Term<'id, ExprHost, ()> for Var<'id> {
    fn lower(self, cx: &mut ScopeCx<'id, '_, ExprHost>) {
        cx.register(None, |child_cx| {
            child_cx.store_mut().push(Op::Var(self.0));
        });
    }
}

/// Assert that a number is close to the expected value within a small epsilon
/// (floating point comparison tolerance).
fn assert_num(v: Num, expected: Num) {
    const EPSILON: Num = 1e-6;
    assert!((v - expected).abs() < EPSILON);
}

impl ExprHost {
    /// Evaluate an expression and assert the result matches the expected value.
    fn eval_assert<TFn, M>(&self, expected: Num, f: TFn)
    where
        TFn: for<'id> FnOnce(&mut Scope<'id, Self>) -> Box<dyn ErasedTerm<'id, Self, M> + 'id>,
    {
        self.scope(f, |o| assert_num(o.value(), expected));
    }
}
