use crate::graph::direction::Undirected;
use crate::graph::graph::Graph;
use crate::graph::ids::{EdgeId, NodeId};

pub type MaximumMatchingGraph = Graph<Undirected, (), ()>;

impl MaximumMatchingGraph {
    #[inline]
    pub fn add_undirected_edge(&mut self, u: NodeId, v: NodeId) -> Option<EdgeId> {
        self.add_edge(u, v, ())
    }
}
