use crate::{
    algorithms::maximum_flow::{edge::MaximumFlowEdge, result::MaxFlowResult, status::Status},
    core::numeric::FlowNum,
    graph::{direction::Directed, graph::Graph, ids::NodeId},
};

pub trait MaximumFlowSolver<N, F: FlowNum> {
    fn solve(&mut self, source: NodeId, sink: NodeId, cut_off: Option<F>) -> Result<MaxFlowResult<F>, Status>;
}

pub trait BuildMaximumFlowSolver<N, F: FlowNum>: Sized {
    fn new(graph: &Graph<Directed, N, MaximumFlowEdge<F>>) -> Self;
}
