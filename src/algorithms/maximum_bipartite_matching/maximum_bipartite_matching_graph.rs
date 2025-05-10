use crate::core::bipartite_graph::BipartiteGraph;
use crate::core::ids::{EdgeId, NodeId};
use crate::prelude::Undirected;

pub type MaximumBipartiteMatchingGraph = BipartiteGraph<Undirected, (), ()>;

impl MaximumBipartiteMatchingGraph {
    #[inline]
    pub fn add_undirected_edge(&mut self, left: NodeId, right: NodeId) -> EdgeId {
        self.add_edge(left, right, ())
    }
}
