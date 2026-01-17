use crate::{
    algorithms::shortest_path::edge::WeightEdge,
    graph::{
        direction::Directed,
        graph::Graph,
        ids::{EdgeId, NodeId},
    },
};
use std::ops::{Deref, DerefMut};

#[derive(Default)]
pub struct ShortestPathGraph<W>(Graph<Directed, (), WeightEdge<W>>);

impl<W> ShortestPathGraph<W> {
    #[inline]
    pub fn add_edge(&mut self, u: NodeId, v: NodeId, weight: W) -> Option<EdgeId> {
        if u.index() >= self.0.num_nodes() || v.index() >= self.0.num_nodes() {
            return None;
        }
        self.0.add_edge(u, v, WeightEdge { weight: weight })
    }
}

impl<F> Deref for ShortestPathGraph<F> {
    type Target = Graph<Directed, (), WeightEdge<F>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<F> DerefMut for ShortestPathGraph<F> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
