use crate::graph::bipartite_graph::BipartiteGraph;
use crate::graph::ids::{EdgeId, NodeId};
use crate::graph::direction::Undirected;

pub type MaximumBipartiteMatchingGraph = BipartiteGraph<Undirected, (), ()>;

impl MaximumBipartiteMatchingGraph {
    #[inline]
    pub fn add_undirected_edge(&mut self, left: NodeId, right: NodeId) -> EdgeId {
        self.add_edge(left, right, ())
    }
}
