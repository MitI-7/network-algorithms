use crate::{
    algorithms::maximum_flow::edge::MaximumFlowEdge,
    graph::{
        direction::Directed,
        graph::Graph,
        ids::{EdgeId, NodeId},
    },
};
use std::ops::{Deref, DerefMut};

pub struct MaximumFlowGraph<F>(Graph<Directed, (), MaximumFlowEdge<F>>);

impl<F> MaximumFlowGraph<F> {
    pub fn new() -> Self {
        Self(Graph::new_directed())
    }

    pub fn add_edge(&mut self, u: NodeId, v: NodeId, upper: F) -> EdgeId {
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

// 相互変換
// impl<F> From<Graph<Directed, (), MaximumFlowEdge<F>>> for MaximumFlowGraph<F> {
//     fn from(g: Graph<Directed, (), MaximumFlowEdge<F>>) -> Self {
//         Self(g)
//     }
// }
// impl<F> From<MaximumFlowGraph<F>> for Graph<Directed, (), MaximumFlowEdge<F>> {
//     fn from(g: MaximumFlowGraph<F>) -> Self {
//         g.0
//     }
// }
