use crate::{
    algorithms::maximum_flow::{edge::MaximumFlowEdge, result::MaxFlowResult, status::Status},
    core::numeric::FlowNum,
    graph::{direction::Directed, graph::Graph, ids::NodeId},
};

pub trait MaximumFlowSolver<N, F: FlowNum> {
    fn new(graph: &Graph<Directed, N, MaximumFlowEdge<F>>) -> Self;
    fn solve(&mut self, source: NodeId, sink: NodeId) -> Result<MaxFlowResult<F>, Status>;
}
