use crate::minimum_cost_flow::residual_network::ResidualNetwork;
use crate::{
    algorithms::minimum_cost_flow::{
        edge::MinimumCostFlowEdge, node::MinimumCostFlowNode, result::MinimumCostFlowResult, status::Status,
    },
    core::numeric::CostNum,
    graph::{direction::Directed, graph::Graph, ids::NodeId},
};

pub fn validate_balance<F: CostNum>(rn: &ResidualNetwork<F>) -> Result<(), Status> {
    if rn.excesses.iter().fold(F::zero(), |sum, &e| sum + e) != F::zero() {
        return Err(Status::Unbalanced);
    }
    Ok(())
}

pub fn validate_infeasible<F: CostNum>(rn: &ResidualNetwork<F>) -> Result<(), Status> {
    if rn.num_edges == 0 {
        if rn.excesses.iter().any(|&e| e != F::zero()) {
            return Err(Status::Infeasible);
        }
    }
    Ok(())
}

pub fn trivial_solution_if_any<F: CostNum>(
    rn: &ResidualNetwork<F>,
) -> Option<Result<MinimumCostFlowResult<F>, Status>> {
    if rn.num_nodes == 0 {
        return Some(Ok(MinimumCostFlowResult {
            objective_value: F::zero(),
            flows: vec![F::zero(); rn.num_edges_original_graph],
        }));
    }

    if rn.num_edges == 0 {
        return Some(Ok(MinimumCostFlowResult {
            objective_value: F::zero(),
            flows: vec![F::zero(); rn.num_edges_original_graph],
        }));
    }

    None
}
