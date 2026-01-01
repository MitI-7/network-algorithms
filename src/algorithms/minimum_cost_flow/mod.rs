pub mod edge;
pub mod node;
mod normalized_network;
pub mod residual_network;
pub mod result;
pub mod solver;
pub mod status;
pub mod successive_shortest_path;

pub use self::status::Status;
// use crate::core::numeric::CostNum;
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
