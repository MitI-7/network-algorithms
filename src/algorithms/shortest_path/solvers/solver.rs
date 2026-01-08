use crate::{
    algorithms::shortest_path::{edge::WeightEdge, status::Status},
    core::numeric::FlowNum,
    graph::{direction::Directed, graph::Graph, ids::NodeId},
};

pub trait ShortestPathSolver<W: FlowNum> {
    fn new(graph: &Graph<Directed, (), WeightEdge<W>>) -> Self;
    fn solve(&mut self, source: NodeId) -> Result<(), Status>;
    fn distance(&self, u: NodeId) -> Option<W>;
    fn reached(&self, u: NodeId) -> bool;
}
