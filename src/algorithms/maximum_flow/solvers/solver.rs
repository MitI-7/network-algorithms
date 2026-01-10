use crate::{
    algorithms::maximum_flow::{edge::MaximumFlowEdge, error::MaximumFlowError},
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
    fn solve(&mut self, source: NodeId, sink: NodeId) -> Result<F, MaximumFlowError>;
    fn flow(&self, u: EdgeId) -> Result<F, MaximumFlowError>;
    fn flows(&self) -> Result<Vec<F>, MaximumFlowError>;
    fn minimum_cut(&mut self) -> Result<Vec<bool>, MaximumFlowError>;
}
