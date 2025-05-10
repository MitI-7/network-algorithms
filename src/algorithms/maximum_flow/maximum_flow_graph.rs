use crate::core::graph::Graph;
use crate::core::direction::Directed;
use crate::edge::capacity::CapEdge;
use crate::core::ids::{NodeId, EdgeId};
use crate::traits::Zero;

pub type MaximumFlowGraph<F = i64>  = Graph<Directed, (), CapEdge<F>>;

impl<F> MaximumFlowGraph<F>
where
    F: Zero,
{
    #[inline]
    pub fn add_directed_edge(&mut self, u: NodeId, v: NodeId, capacity: F) -> EdgeId {
        self.add_edge(u, v, CapEdge { flow: F::zero(), upper: capacity })
    }
}