use crate::algorithms::minimum_cost_flow::edge::MinimumCostFlowEdge;
use crate::algorithms::minimum_cost_flow::node::MinimumCostFlowNode;
use crate::algorithms::minimum_cost_flow::{MinimumCostFlowNum, Status};
use crate::graph::direction::Directed;
use crate::graph::graph::Graph;
use crate::algorithms::minimum_cost_flow::result::MinimumCostFlowResult;

pub trait MinimumCostFlowSolver<F>
where
    F: MinimumCostFlowNum,
{
    fn solve(
        &mut self,
        graph: &Graph<Directed, MinimumCostFlowNode<F>, MinimumCostFlowEdge<F>>,
    ) -> Result<MinimumCostFlowResult<F>, Status>;
}
