//! Core abstraction for modeling an algebraic problem.

use crate::Constraint;

/// Newtype wrapper around a key used as a [`System`] variable.
#[derive(Debug, Clone, PartialEq, Eq, Hash, derive_more::Display, derive_more::From)]
#[display("var:{_0}")]
pub struct Var<K>(K);

impl<K> Var<K> {
    /// Create a new [`Var`] with the given key.
    #[must_use = "constructor"]
    pub const fn new(key: K) -> Self {
        Self(key)
    }
}

impl<K> Var<K> {
    /// Unwrap the inner key.
    #[must_use = "unwraps the inner key"]
    pub fn into_key(self) -> K {
        let Self(key) = self;
        key
    }

    /// Borrow the inner key.
    #[must_use = "getter"]
    pub const fn key(&self) -> &K {
        let Self(key) = self;
        key
    }

    /// Map a function over the inner key.
    #[must_use = "mapping moves the original value"]
    pub fn map<O>(self, f: impl FnOnce(K) -> O) -> Var<O> {
        let Self(key) = self;
        Var::new(f(key))
    }
}

/// Create a new [`Var`] with the given key.
#[must_use = "constructor"]
pub const fn var<K>(key: K) -> Var<K> {
    Var::new(key)
}

/// A collection of [`Constraint`]s.
/// This is the core abstraction of this library.
///
/// A `System` represents an abstract problem definition, where a number
/// of *variables* are related to each other via algebraic constraints.
///
/// Variables are labels `K` derived from constraints, that uniquely
/// identify an algebraic term which must resolve to some value `V`.
/// Constraints model the relationships between these terms.
#[derive(Debug, Clone)]
pub struct System<K, V> {
    /// Variable relationships modeling the problem.
    constraints: Vec<Constraint<K, V>>,
}

impl<K, V> Default for System<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> System<K, V> {
    /// Create a new, empty [`System`].
    #[must_use = "constructor"]
    pub const fn new() -> Self {
        Self {
            constraints: Vec::new(),
        }
    }

    /// Add a [`Constraint`] to the [`System`].
    pub fn push(&mut self, constraint: Constraint<K, V>) {
        self.constraints.push(constraint);
    }

    /// Add multiple [`Constraint`]s to the [`System`].
    pub fn extend<I>(&mut self, constraints: I)
    where
        I: IntoIterator<Item = Constraint<K, V>>,
    {
        self.constraints.extend(constraints);
    }
}

impl<K, V> FromIterator<Constraint<K, V>> for System<K, V> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Constraint<K, V>>,
    {
        let constraints = iter.into_iter().collect();
        Self { constraints }
    }
}
