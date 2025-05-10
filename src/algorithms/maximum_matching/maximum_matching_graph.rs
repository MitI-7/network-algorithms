use crate::core::graph::Graph;
use crate::core::ids::{EdgeId, NodeId};
use crate::prelude::Undirected;

pub type MaximumMatchingGraph = Graph<Undirected, (), ()>;

impl MaximumMatchingGraph {
    #[inline]
    pub fn add_undirected_edge(&mut self, u: NodeId, v: NodeId) -> EdgeId {
        self.add_edge(u, v, ())
    }
}
