//! Defines a [`Constraint`] applied to a [`System`].
//!
//! [`System`]: crate::System

use super::system::Var;

/// An algebraic relationship expressed between variables in a [`System`].
///
/// [`System`]: crate::System
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Constraint<K, V> {
    /// Feed known values for specified variables.
    Seed(Seed<K, V>),
    /// Operate on linear equations.
    LinearEq(LinearEq<K, V>),
    /// Compute the absolute difference between two variables.
    AbsDiff(AbsDiff<K, V>),
    /// Choose the minimum of two variables.
    Min(Min<K>),
    /// Choose the maximum of two variables.
    Max(Max<K>),
}

/// A [`Constraint`] that assigns a fixed `value` to the `target` variable.
///
/// Applying such a constraint "seeds" the system with input data.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Seed<K, V> {
    /// The variable to seed.
    pub target: Var<K>,
    /// A known value.
    pub value: V,
}

/// A term in a [`LinearEq`].
///
/// Each term is a product of the `target` variable and a coefficient
/// `coeff`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LinearEqTerm<K, V> {
    /// The variable associated with this term.
    pub target: Var<K>,
    /// The coefficient, by which the `target` is multiplied.
    pub coeff: V,
}

/// A [`Constraint`] modeling a standard-form linear equation.
/// Note that the equation does not have to be normalized (i.e. may contain
/// multiple terms with the same variable).
///
/// It consists of [`LinearEqTerm`]s and a known value, which the terms
/// must sum up to.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LinearEq<K, V> {
    /// Terms of the equation.
    pub var_terms: Vec<LinearEqTerm<K, V>>,
    /// A known value the terms must sum up to.
    pub const_term: V,
}

/// A [`Constraint`] that assigns the absolute difference between two
/// variables (`lhs` and `rhs`) to the `target`.
///
/// This relationship is only interpreted one-way.
/// A known value of the target cannot determine the values of the input
/// variables.
///
/// This operation additionally models a coefficient `coeff` applied to
/// `lhs` and `rhs` before taking the absolute value.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AbsDiff<K, V> {
    /// This variable receives the computed value.
    pub target: Var<K>,
    /// Left-hand side variable.
    pub lhs: Var<K>,
    /// Right-hand side variable.
    pub rhs: Var<K>,
    /// Coefficient applied to `lhs` and `rhs` before taking the absolute
    /// value.
    pub coeff: V,
}

/// A [`Constraint`] that assigns the minimum of two variables (`lhs` and
/// `rhs`) to the `target`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Min<K> {
    /// This variable receives the computed value.
    pub target: Var<K>,
    /// Left-hand side variable.
    pub lhs: Var<K>,
    /// Right-hand side variable.
    pub rhs: Var<K>,
}

/// A [`Constraint`] that assigns the maximum of two variables (`lhs` and
/// `rhs`) to the `target`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Max<K> {
    /// This variable receives the computed value.
    pub target: Var<K>,
    /// Left-hand side variable.
    pub lhs: Var<K>,
    /// Right-hand side variable.
    pub rhs: Var<K>,
}
