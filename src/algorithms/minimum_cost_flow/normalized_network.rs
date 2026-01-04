use crate::{
    algorithms::minimum_cost_flow::{
        edge::MinimumCostFlowEdge, node::MinimumCostFlowNode,
    },
    graph::{direction::Directed, graph::Graph, ids::NodeId},
    core::numeric::CostNum,
};

#[derive(Clone, Copy, Debug)]
pub struct NormalizedEdge<F> {
    pub u: NodeId,
    pub v: NodeId,
    pub lower: F,
    pub upper: F,          // original.upper - original.lower
    pub cost: F,           // non-negative
    pub is_reversed: bool,
}

pub struct NormalizedNetwork<'a, F>
where
    F: CostNum,
{
    base: &'a Graph<Directed, MinimumCostFlowNode<F>, MinimumCostFlowEdge<F>>,
    b: Vec<F>,
}

impl<'a, F> NormalizedNetwork<'a, F>
where
    F: CostNum,
{
    pub fn new(base: &'a Graph<Directed, MinimumCostFlowNode<F>, MinimumCostFlowEdge<F>>) -> Self {
        let n = base.num_nodes();
        let mut b = Vec::with_capacity(n);
        for u in 0..n {
            b.push(base.get_node(NodeId(u)).unwrap().data.b);
        }

        for e in base.edges() {
            let u = e.u.index();
            let v = e.v.index();
            let lower = e.data.lower;
            let upper = e.data.upper;
            let cost = e.data.cost;

            if cost >= F::zero() {
                b[u] = b[u] - lower;
                b[v] = b[v] + lower;
            } else {
                b[u] = b[u] - upper;
                b[v] = b[v] + upper;
            }
        }

        Self { base, b }
    }

    pub fn num_nodes(&self) -> usize {
        self.base.num_nodes()
    }

    pub fn num_edges(&self) -> usize {
        self.base.num_edges()
    }

    pub fn excesses(&self) -> &[F] {
        &self.b
    }

    pub fn iter_edges(&self) -> impl Iterator<Item = NormalizedEdge<F>> + '_ {
        self.base.edges().map(|edge| {
            let (mut u, mut v) = (edge.u, edge.v);
            let mut cost = edge.data.cost;
            let upper = edge.data.upper - edge.data.lower;
            let mut is_reversed = false;

            if cost < F::zero() {
                (u, v) = (v, u);
                cost = -cost;
                is_reversed = true;
            }

            debug_assert!(cost >= F::zero());
            debug_assert!(upper >= F::zero());

            NormalizedEdge {
                u,
                v,
                lower: edge.data.lower,
                upper,
                cost,
                is_reversed,
            }
        })
    }
}
