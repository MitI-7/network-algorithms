use crate::{
    algorithms::maximum_flow::edge::MaximumFlowEdge,
    graph::{
        direction::Directed,
        graph::Graph,
        ids::{EdgeId, NodeId},
    },
};
use std::ops::{Deref, DerefMut};

#[derive(Default)]
pub struct MaximumFlowGraph<F>(Graph<Directed, (), MaximumFlowEdge<F>>);

impl<F> MaximumFlowGraph<F> {
    pub fn add_edge(&mut self, u: NodeId, v: NodeId, upper: F) -> Option<EdgeId> {
        if u.index() >= self.0.num_nodes() || v.index() >= self.0.num_nodes() {
            return None;
        }
        self.0.add_edge(u, v, MaximumFlowEdge { upper })
    }
}

impl<F> Deref for MaximumFlowGraph<F> {
    type Target = Graph<Directed, (), MaximumFlowEdge<F>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<F> DerefMut for MaximumFlowGraph<F> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
