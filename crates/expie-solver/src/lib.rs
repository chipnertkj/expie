#![doc = include_str!("../README.md")]

mod constraint;
mod system;

pub use self::{
    constraint::{AbsDiff, Constraint, LinearEq, Max, Min, Seed},
    system::{System, Var, var},
};
