use crate::graph::{
    direction::Undirected,
    graph::Graph,
    ids::{EdgeId, NodeId},
};
use std::ops::{Deref, DerefMut};

#[derive(Default)]
pub struct MaximumMatchingGraph(Graph<Undirected, (), ()>);

impl MaximumMatchingGraph {
    #[inline]
    pub fn add_edge(&mut self, u: NodeId, v: NodeId) -> Option<EdgeId> {
        if u.index() >= self.0.num_nodes() || v.index() >= self.0.num_nodes() {
            return None;
        }
        self.0.add_edge(u, v, ())
    }
}

impl Deref for MaximumMatchingGraph {
    type Target = Graph<Undirected, (), ()>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MaximumMatchingGraph {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
