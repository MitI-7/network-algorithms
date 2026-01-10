use crate::{
    algorithms::minimum_cost_flow::{
        edge::MinimumCostFlowEdge, error::MinimumCostFlowError, node::MinimumCostFlowNode,
    },
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
    fn solve(&mut self) -> Result<F, MinimumCostFlowError>;
    fn flow(&self, edge_id: EdgeId) -> Result<F, MinimumCostFlowError>;
    fn flows(&self) -> Result<Vec<F>, MinimumCostFlowError>;
    fn potential(&self, node_id: NodeId) -> Result<F, MinimumCostFlowError>;
    fn potentials(&self) -> Result<Vec<F>, MinimumCostFlowError>;
}
