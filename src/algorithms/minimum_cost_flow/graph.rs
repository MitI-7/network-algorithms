use crate::{
    algorithms::minimum_cost_flow::{edge::MinimumCostFlowEdge, node::MinimumCostFlowNode},
    graph::{
        direction::Directed,
        graph::Graph,
        ids::{EdgeId, NodeId},
    },
};
use std::ops::{Deref, DerefMut};

pub struct MinimumCostFlowGraph<F>(Graph<Directed, MinimumCostFlowNode<F>, MinimumCostFlowEdge<F>>);

impl<F> MinimumCostFlowGraph<F>
where
    F: Default,
{
    pub fn new() -> Self {
        Self(Graph::new_directed())
    }

    pub fn add_edge(
        &mut self,
        u: NodeId,
        v: NodeId,
        lower: F,
        upper: F,
        cost: F,
    ) -> Option<EdgeId> {
        if u.index() >= self.num_nodes() || v.index() >= self.num_nodes() {
            return None;
        }
        self.0
            .add_edge(u, v, MinimumCostFlowEdge { lower, upper, cost })
    }

    pub fn set_excess(&mut self, u: NodeId, b: F) {
        self.get_node_mut(u).unwrap().data.b = b;
    }
}

impl<F> Deref for MinimumCostFlowGraph<F> {
    type Target = Graph<Directed, MinimumCostFlowNode<F>, MinimumCostFlowEdge<F>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<F> DerefMut for MinimumCostFlowGraph<F> {
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
