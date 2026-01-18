use crate::{
    Edge, Node,
    core::numeric::CostNum,
    graph::{direction::Directed, graph::Graph, ids::NodeId},
};

#[derive(Clone, Copy, Debug)]
pub struct NormalizedEdge<F> {
    pub u: NodeId,
    pub v: NodeId,
    pub lower: F,
    pub upper: F, // original.upper - original.lower
    pub cost: F,  // non-negative
    pub is_reversed: bool,
}

pub struct NormalizedNetwork<'a, F, N, E, LF, UF, CF, BF>
where
    F: CostNum,
{
    base: &'a Graph<Directed, N, E>,
    b: Vec<F>,
    lower_fn: LF,
    upper_fn: UF,
    cost_fn: CF,
    _b_fn: BF,
}

impl<'a, F, N, E, LF, UF, CF, BF> NormalizedNetwork<'a, F, N, E, LF, UF, CF, BF>
where
    F: CostNum,
    LF: Fn(&Edge<E>) -> F,
    UF: Fn(&Edge<E>) -> F,
    CF: Fn(&Edge<E>) -> F,
    BF: Fn(&Node<N>) -> F,
{
    pub fn from(base: &'a Graph<Directed, N, E>, lower_fn: LF, upper_fn: UF, cost_fn: CF, b_fn: BF) -> Self {
        let n = base.num_nodes();
        let mut b = Vec::with_capacity(n);
        for u in 0..n {
            b.push(b_fn(base.get_node(NodeId(u)).unwrap()));
        }

        for e in base.edges() {
            let u = e.u.index();
            let v = e.v.index();
            let lower = lower_fn(e);
            let upper = upper_fn(e);
            let cost = cost_fn(e);

            if cost >= F::zero() {
                b[u] -= lower;
                b[v] += lower;
            } else {
                b[u] -= upper;
                b[v] += upper;
            }
        }

        Self { base, b, lower_fn, upper_fn, cost_fn, _b_fn: b_fn }
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
            let mut cost = (self.cost_fn)(edge);
            let upper = (self.upper_fn)(edge) - (self.lower_fn)(edge);
            let mut is_reversed = false;

            if cost < F::zero() {
                (u, v) = (v, u);
                cost = -cost;
                is_reversed = true;
            }

            debug_assert!(cost >= F::zero());
            debug_assert!(upper >= F::zero());

            NormalizedEdge { u, v, lower: (self.lower_fn)(edge), upper, cost, is_reversed }
        })
    }
}
