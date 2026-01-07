use crate::{
    algorithms::minimum_cost_flow::{edge::MinimumCostFlowEdge, node::MinimumCostFlowNode},
    graph::{
        direction::Directed,
        graph::Graph,
        ids::{EdgeId, NodeId},
    },
};
use std::ops::{Deref, DerefMut};

#[derive(Default)]
pub struct MinimumCostFlowGraph<F>(Graph<Directed, MinimumCostFlowNode<F>, MinimumCostFlowEdge<F>>);

impl<F> MinimumCostFlowGraph<F> {
    pub fn add_edge(&mut self, u: NodeId, v: NodeId, lower: F, upper: F, cost: F) -> Option<EdgeId> {
        if u.index() >= self.num_nodes() || v.index() >= self.num_nodes() {
            return None;
        }
        self.0.add_edge(u, v, MinimumCostFlowEdge { lower, upper, cost })
    }

    pub fn set_excess(&mut self, u: NodeId, b: F) -> Option<()> {
        let node = self.get_node_mut(u)?;
        node.data.b = b;
        Some(())
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
