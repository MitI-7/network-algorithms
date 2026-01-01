use crate::algorithms::maximum_flow::result::MaxFlowResult;
use crate::algorithms::maximum_flow::{edge::MaximumFlowEdge, status::Status};
use crate::core::numeric::FlowNum;
use crate::graph::{direction::Directed, graph::Graph, ids::NodeId};

pub trait MaximumFlowSolver<F>
where
    F: FlowNum,
{
    fn solve(
        &mut self,
        graph: &Graph<Directed, (), MaximumFlowEdge<F>>,
        source: NodeId,
        sink: NodeId,
        upper: Option<F>,
    ) -> Result<MaxFlowResult<F>, Status>;

    fn minimum_cut(
        &mut self,
        graph: &Graph<Directed, (), MaximumFlowEdge<F>>,
        source: NodeId,
        sink: NodeId,
        upper: Option<F>,
    ) -> Result<MaxFlowResult<F>, Status>;
}
