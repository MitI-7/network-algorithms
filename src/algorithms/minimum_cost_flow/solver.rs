use crate::{
    algorithms::minimum_cost_flow::{
        edge::MinimumCostFlowEdge, node::MinimumCostFlowNode, result::MinimumCostFlowResult,
        status::Status,
    },
    core::numeric::CostNum,
    graph::{direction::Directed, graph::Graph},
};

pub trait MinimumCostFlowSolver<F>
where
    F: CostNum,
{
    fn solve(
        &mut self,
        graph: &Graph<Directed, MinimumCostFlowNode<F>, MinimumCostFlowEdge<F>>,
    ) -> Result<MinimumCostFlowResult<F>, Status>;
}
