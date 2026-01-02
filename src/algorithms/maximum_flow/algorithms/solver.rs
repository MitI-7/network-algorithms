use crate::algorithms::maximum_flow::result::MaxFlowResult;
use crate::algorithms::maximum_flow::{edge::MaximumFlowEdge, status::Status};
use crate::core::numeric::FlowNum;
use crate::graph::{direction::Directed, graph::Graph, ids::NodeId};

pub trait MaximumFlowSolver<N, F>
where
    F: FlowNum,
{
    type Prepared;
    fn solve(
        &mut self,
        graph: &Graph<Directed, N, MaximumFlowEdge<F>>,
        source: NodeId,
        sink: NodeId,
        cut_off: Option<F>,
    ) -> Result<MaxFlowResult<F>, Status>;

    fn prepare(
        &mut self,
        graph: &Graph<Directed, N, MaximumFlowEdge<F>>,
    ) -> Result<Self::Prepared, Status>;

    fn solve_with_prepared(
        &mut self,
        prepared: &Self::Prepared,
        s: NodeId,
        t: NodeId,
        cut_off: Option<F>,
    ) -> Result<MaxFlowResult<F>, Status>;
}
