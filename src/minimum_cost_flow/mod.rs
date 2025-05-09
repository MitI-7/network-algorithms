// pub mod cost_scaling_push_relabel;
pub mod csr;
pub mod translater;
// pub mod cycle_canceling;
// pub mod dual_network_simplex;
// pub mod network_simplex_pivot_rules;
// pub mod out_of_kilter;
// pub mod parametric_network_simplex;
pub mod primal_dual;
// pub mod primal_network_simplex;
// pub mod spanning_tree_structure;
pub mod status;
// pub mod successive_shortest_path;
use crate::core::graph::Graph;
use crate::core::direction::Directed;
use crate::edge::capacity_cost::CapCostEdge;
use crate::node::excess::ExcessNode;

pub trait MinimumCostFlowSolver<Flow>
where
    Flow: MinimumCostFlowNum,
{
    fn solve(&mut self, graph: &mut Graph<Directed, ExcessNode<Flow>, CapCostEdge<Flow>>) -> Result<Flow, Status>;
}

use crate::traits::{One, Zero};
use std::fmt::Debug;
use std::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};
// pub use self::cost_scaling_push_relabel::CostScalingPushRelabel;
// pub use self::cycle_canceling::CycleCanceling;
// pub use self::dual_network_simplex::DualNetworkSimplex;
// pub use crate::graph::minimum_cost_flow_graph::Graph;
// pub use self::out_of_kilter::OutOfKilter;
// pub use self::parametric_network_simplex::ParametricNetworkSimplex;
pub use self::primal_dual::PrimalDual;
// pub use self::primal_network_simplex::PrimalNetworkSimplex;
pub use self::status::Status;
// pub use self::successive_shortest_path::SuccessiveShortestPath;

// pub trait FlowNum = Zero + Ord + Add<Output = Self> + Sub<Output = Self> + AddAssign + SubAssign + Copy;
// Flow: FlowNum + Neg<Output = Flow> + std::ops::Mul<Output = Flow> + One,
pub trait MinimumCostFlowNum:
    Zero + Ord + Add<Output = Self> + Sub<Output = Self> + Neg<Output = Self> + Mul<Output = Self> + AddAssign + SubAssign + Clone + Copy + One + Debug + Default
{
}
impl<T> MinimumCostFlowNum for T where
    T: Zero + Ord + Add<Output = Self> + Sub<Output = Self> + Neg<Output = Self> + Mul<Output = Self> + AddAssign + SubAssign + Clone + Copy + One + Debug + Default
{
}
