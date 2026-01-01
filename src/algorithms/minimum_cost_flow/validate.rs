use crate::{
    algorithms::minimum_cost_flow::{
        MinimumCostFlowNum, Status, edge::MinimumCostFlowEdge, node::MinimumCostFlowNode,
        result::MinimumCostFlowResult,
    },
    graph::{direction::Directed, graph::Graph, ids::NodeId},
};

pub fn validate<F: MinimumCostFlowNum>(
    graph: &mut Graph<Directed, MinimumCostFlowNode<F>, MinimumCostFlowEdge<F>>,
) -> Result<(), Status> {
    if (0..graph.num_nodes())
        .into_iter()
        .fold(F::zero(), |sum, u| sum + graph.get_node(NodeId(u)).data.b)
        != F::zero()
    {
        return Err(Status::Unbalanced);
    }

    let mut t = F::zero();
    for u in 0..graph.num_nodes() {
        t += graph.get_node(NodeId(u)).data.b;
    }
    if t != F::zero() {
        return Err(Status::Unbalanced);
    }

    if graph.num_edges() == 0 {
        for u in 0..graph.num_nodes() {
            if graph.get_node(NodeId(u)).data.b != F::zero() {
                return Err(Status::Infeasible);
            }
        }
    }

    Ok(())
}

pub fn trivial<F: MinimumCostFlowNum>(
    graph: &mut Graph<Directed, MinimumCostFlowNode<F>, MinimumCostFlowEdge<F>>,
) -> Option<Result<MinimumCostFlowResult<F>, Status>> {
    if graph.num_nodes() == 0 {
        return Some(Ok(MinimumCostFlowResult {
            objective_value: F::zero(),
            flows: vec![F::zero(); graph.num_edges()],
        }));
    }

    if graph.num_edges() == 0 {
        for u in 0..graph.num_nodes() {
            if graph.get_node(NodeId(u)).data.b != F::zero() {
                return Some(Err(Status::Infeasible));
            }
        }
        return Some(Ok(MinimumCostFlowResult {
            objective_value: F::zero(),
            flows: vec![F::zero(); graph.num_edges()],
        }));
    }

    None
}
