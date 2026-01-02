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
    pub upper: F,          // 正規化後の上限 (= original.upper - original.lower)
    pub cost: F,           // non-negative
    pub edge_index: usize, // 元の graph.edges の index
}

pub struct NormalizedNetwork<'a, F>
where
    F: CostNum,
{
    base: &'a Graph<Directed, MinimumCostFlowNode<F>, MinimumCostFlowEdge<F>>,
    b: Vec<F>, // 正規化後の供給需要
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

        // lower除去 + cost非負化（負コスト反転）に対応した b 更新
        for e in base.edges() {
            let u = e.u.index();
            let v = e.v.index();
            let lower = e.data.lower;
            let upper = e.data.upper;
            let cost = e.data.cost;

            if cost >= F::zero() {
                // x = lower + y
                b[u] = b[u] - lower;
                b[v] = b[v] + lower;
            } else {
                // x = upper - y   （負コスト辺は上限まで流して反転）
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
        self.base.edges().enumerate().map(|(i, e)| {
            let mut u = e.u;
            let mut v = e.v;
            let mut cost = e.data.cost;
            let upper = e.data.upper - e.data.lower;

            if cost < F::zero() {
                (u, v) = (v, u);
                cost = -cost;
            }

            debug_assert!(cost >= F::zero());
            debug_assert!(upper >= F::zero());

            NormalizedEdge {
                u,
                v,
                upper,
                cost,
                edge_index: i,
            }
        })
    }
}
