use std::fmt::Debug;
use std::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};
use crate::traits::{One, Zero};

// pub mod csr;
// pub mod highest_gain_path;
// pub mod primal_dual;
// pub mod primal_dual_push_relabel;
pub mod status;
pub mod generalized_maximum_flow_graph;

pub trait GeneralizedMaximumFlowNum:
Zero + Ord + Add<Output = Self> + Sub<Output = Self> + Neg<Output = Self> + Mul<Output = Self> + AddAssign + SubAssign + Clone + Copy + One + Debug + Default
{
}
impl<T> GeneralizedMaximumFlowNum for T where
    T: Zero + Ord + Add<Output = Self> + Sub<Output = Self> + Neg<Output = Self> + Mul<Output = Self> + AddAssign + SubAssign + Clone + Copy + One + Debug + Default
{
}
