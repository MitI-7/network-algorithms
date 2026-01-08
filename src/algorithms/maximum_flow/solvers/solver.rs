use crate::{
    algorithms::maximum_flow::{edge::MaximumFlowEdge, status::Status},
    core::numeric::FlowNum,
    graph::{
        direction::Directed,
        graph::Graph,
        ids::{EdgeId, NodeId},
    },
};

pub trait MaximumFlowSolver<F: FlowNum> {
    fn new<N>(graph: &Graph<Directed, N, MaximumFlowEdge<F>>) -> Self
    where
        Self: Sized;
    fn solve(&mut self, source: NodeId, sink: NodeId) -> Result<F, Status>;
    fn flow(&self, u: EdgeId) -> Option<F>;
    fn minimum_cut(&mut self) -> Result<Vec<bool>, Status>;
}
