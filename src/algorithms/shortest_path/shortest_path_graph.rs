use crate::core::direction::Directed;
use crate::core::graph::Graph;
use crate::core::ids::{EdgeId, NodeId};
use crate::edge::weight::WeightEdge;

pub type ShortestPathGraph<W = i64> = Graph<Directed, (), WeightEdge<W>>;

impl<W> ShortestPathGraph<W> {
    #[inline]
    pub fn add_directed_edge(&mut self, u: NodeId, v: NodeId, weight: W) -> EdgeId {
        self.add_edge(u, v, WeightEdge { weight })
    }
}
