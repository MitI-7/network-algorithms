pub mod cost_scaling_push_relabel;
pub mod csr;
pub mod cycle_canceling;
pub mod dual_network_simplex;
pub mod graph;
pub mod network_simplex_pivot_rules;
pub mod out_of_kilter;
pub mod parametric_network_simplex;
pub mod primal_dual;
pub mod primal_network_simplex;
pub mod spanning_tree_structure;
pub mod status;
pub mod successive_shortest_path;

pub trait MinimumCostFlowSolver<Flow> {
    fn solve(&mut self, graph: &mut graph::Graph<Flow>) -> Result<Flow, status::Status>;
}

pub use self::cost_scaling_push_relabel::CostScalingPushRelabel;
pub use self::cycle_canceling::CycleCanceling;
pub use self::dual_network_simplex::DualNetworkSimplex;
pub use self::graph::Graph;
pub use self::out_of_kilter::OutOfKilter;
pub use self::parametric_network_simplex::ParametricNetworkSimplex;
pub use self::primal_dual::PrimalDual;
pub use self::primal_network_simplex::PrimalNetworkSimplex;
pub use self::status::Status;
pub use self::successive_shortest_path::SuccessiveShortestPath;
