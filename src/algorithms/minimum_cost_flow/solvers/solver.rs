use crate::{
    algorithms::minimum_cost_flow::{edge::MinimumCostFlowEdge, node::MinimumCostFlowNode, status::Status},
    core::numeric::CostNum,
    graph::{direction::Directed, graph::Graph},
    ids::{EdgeId, NodeId},
};

pub trait MinimumCostFlowSolver<F>
where
    F: CostNum,
{
    fn new(graph: &Graph<Directed, MinimumCostFlowNode<F>, MinimumCostFlowEdge<F>>) -> Self
    where
        Self: Sized;
    fn solve(&mut self) -> Result<F, Status>;
    fn flow(&self, edge_id: EdgeId) -> Option<F>;
    fn flows(&self) -> Vec<F>;
    fn potential(&self, node_id: NodeId) -> Option<F>;
    fn potentials(&self) -> Vec<F>;
}
