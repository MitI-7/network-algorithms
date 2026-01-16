use crate::graph::{
    bipartite_graph::BipartiteGraph,
    direction::Undirected,
    ids::{EdgeId, LeftNodeId, RightNodeId},
};
use std::ops::{Deref, DerefMut};

pub struct MaximumBipartiteMatchingGraph(BipartiteGraph<Undirected, (), ()>);

impl MaximumBipartiteMatchingGraph {
    #[inline]
    pub fn add_edge(&mut self, left: LeftNodeId, right: RightNodeId) -> Option<EdgeId> {
        self.0.add_edge(left, right, ())
    }
}

impl Deref for MaximumBipartiteMatchingGraph {
    type Target = BipartiteGraph<Undirected, (), ()>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MaximumBipartiteMatchingGraph {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
