pub mod edge;
pub mod residual_network;
// pub mod network_simplex_pivot_rules;
// pub mod primal_dual;
// pub mod primal_network_simplex;
pub mod solver;
// pub mod spanning_tree_structure;
pub mod node;
mod normalized_network;
pub mod result;
pub mod status;
pub mod successive_shortest_path;
// pub mod translater;

use crate::core::numeric::CostNum;


// pub use self::primal_dual::PrimalDual;
pub use self::status::Status;
use crate::algorithms::minimum_cost_flow::edge::MinimumCostFlowEdge;
use num_traits::{One, Zero};
use std::fmt::Debug;
use std::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};

//
// pub trait FlowNum = Zero + Ord + Add<Output = Self> + Sub<Output = Self> + AddAssign + SubAssign + Copy;
// Flow: FlowNum + Neg<Output = Flow> + std::ops::Mul<Output = Flow> + One,
pub trait MinimumCostFlowNum:
    Zero
    + Ord
    + Add<Output = Self>
    + Sub<Output = Self>
    + Neg<Output = Self>
    + Mul<Output = Self>
    + AddAssign
    + SubAssign
    + Clone
    + Copy
    + One
    + Debug
    + Default
{
}
impl<T> MinimumCostFlowNum for T where
    T: Zero
        + Ord
        + Add<Output = Self>
        + Sub<Output = Self>
        + Neg<Output = Self>
        + Mul<Output = Self>
        + AddAssign
        + SubAssign
        + Clone
        + Copy
        + One
        + Debug
        + Default
{
}
