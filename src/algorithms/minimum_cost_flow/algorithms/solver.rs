use crate::{
    algorithms::minimum_cost_flow::{
        edge::MinimumCostFlowEdge, node::MinimumCostFlowNode, result::MinimumCostFlowResult, status::Status,
    },
    core::numeric::CostNum,
    graph::{direction::Directed, graph::Graph},
};

pub trait MinimumCostFlowSolver<F>
where
    F: CostNum,
{
    fn new(graph: &Graph<Directed, MinimumCostFlowNode<F>, MinimumCostFlowEdge<F>>) -> Self;
    fn solve(&mut self) -> Result<MinimumCostFlowResult<F>, Status>;
}
