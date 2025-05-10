use crate::core::graph::Graph;
use crate::core::direction::Directed;
use crate::edge::capacity_cost::CapCostEdge;
use crate::core::ids::{NodeId, EdgeId};
use crate::traits::Zero;
use crate::node::excess::ExcessNode;

pub type MinimumCostFlowGraph<F = i64>  = Graph<Directed, ExcessNode<F>, CapCostEdge<F>>;

impl<F> MinimumCostFlowGraph<F>
where
    F: Zero + Default,
{
    #[inline]
    pub fn add_directed_edge(&mut self, u: NodeId, v: NodeId, lower: F, upper: F, cost: F) -> EdgeId {
        self.add_edge(u, v, CapCostEdge { flow: F::zero(), lower, upper, cost })
    }
}