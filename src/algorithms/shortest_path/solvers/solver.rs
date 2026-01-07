use crate::{
    algorithms::shortest_path::{edge::WeightEdge, result::ShortestPathResult, status::Status},
    core::numeric::FlowNum,
    graph::{direction::Directed, graph::Graph, ids::NodeId},
};

pub trait ShortestPathSolver<W: FlowNum> {
    fn new(graph: &Graph<Directed, (), WeightEdge<W>>) -> Self;
    fn solve(&mut self, source: NodeId) -> Result<ShortestPathResult<W>, Status>;
}
