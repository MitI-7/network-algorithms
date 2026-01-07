use crate::algorithms::shortest_path::edge::WeightEdge;
use crate::graph::direction::Directed;
use crate::graph::graph::Graph;
use crate::graph::ids::{EdgeId, NodeId};

pub type ShortestPathGraph<W = i64> = Graph<Directed, (), WeightEdge<W>>;

impl<W> ShortestPathGraph<W> {
    #[inline]
    pub fn add_directed_edge(&mut self, u: NodeId, v: NodeId, weight: W) -> Option<EdgeId> {
        self.add_edge(u, v, WeightEdge { weight })
    }
}
