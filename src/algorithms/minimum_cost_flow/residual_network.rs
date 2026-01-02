use crate::graph::ids::ArcId;
use crate::graph::iter::ArcIdRange;
use crate::{
    algorithms::minimum_cost_flow::{
        edge::MinimumCostFlowEdge,
        node::MinimumCostFlowNode,
        normalized_network::{NormalizedEdge, NormalizedNetwork},
    },
    graph::{
        direction::Directed,
        graph::Graph,
        ids::{EdgeId, NodeId},
    },
    core::numeric::CostNum,
};
use std::{cmp::Reverse, collections::BinaryHeap};

#[derive(Default)]
pub(crate) struct ResidualNetwork<F> {
    pub(crate) num_nodes: usize,
    pub(crate) num_edges: usize,
    pub(crate) edge_id_to_arc_id: Box<[ArcId]>,

    pub(crate) excesses: Box<[F]>,
    pub(crate) potentials: Box<[F]>,

    pub(crate) start: Box<[usize]>,
    pub(crate) to: Box<[NodeId]>,
    pub(crate) flow: Box<[F]>,
    pub(crate) upper: Box<[F]>,
    pub(crate) cost: Box<[F]>,
    pub(crate) rev: Box<[ArcId]>,

    pub(crate) is_reversed: Box<[bool]>,
}

impl<F> ResidualNetwork<F>
where
    F: CostNum,
{
    pub fn build(
        &mut self,
        graph: &NormalizedNetwork<'_, F>,
        artificial_nodes: Option<&[NodeId]>,
        artificial_edges: Option<&[NormalizedEdge<F>]>,
        fix_excesses: Option<&[F]>,
    ) {
        if graph.num_nodes() == 0 {
            return;
        }

        self.num_nodes = graph.num_nodes() + artificial_nodes.unwrap_or(&[]).len();
        self.num_edges = graph.num_edges() + artificial_edges.unwrap_or(&[]).len();

        // b は正規化後のものを使う
        self.excesses = vec![F::zero(); self.num_nodes].into_boxed_slice();
        for (u, e) in graph.excesses().iter().enumerate() {
            self.excesses[u] = *e;
        }

        if fix_excesses.is_some() {
            for u in 0..self.num_nodes {
                self.excesses[u] += fix_excesses.unwrap()[u];
            }
        }

        self.edge_id_to_arc_id = vec![ArcId(usize::MAX); self.num_edges].into_boxed_slice();
        self.start = vec![0; self.num_nodes + 1].into_boxed_slice();
        self.to = vec![NodeId(usize::MAX); self.num_edges * 2].into_boxed_slice();
        self.flow = vec![F::zero(); self.num_edges * 2].into_boxed_slice();
        self.upper = vec![F::zero(); self.num_edges * 2].into_boxed_slice();
        self.cost = vec![F::zero(); self.num_edges * 2].into_boxed_slice();
        self.rev = vec![ArcId(usize::MAX); self.num_edges * 2].into_boxed_slice();
        self.potentials = vec![F::zero(); self.num_nodes].into_boxed_slice();

        let mut degree = vec![0usize; self.num_nodes];

        for ne in graph
            .iter_edges()
            .chain(artificial_edges.into_iter().flatten().copied())
        {
            degree[ne.u.index()] += 1;
            degree[ne.v.index()] += 1;
        }

        // start の prefix sum
        for u in 1..=self.num_nodes {
            self.start[u] = self.start[u - 1] + degree[u - 1];
        }

        let mut counter = vec![0usize; self.num_nodes];

        for edge in graph
            .iter_edges()
            .chain(artificial_edges.into_iter().flatten().copied())
        {
            // ここでは lower は常に 0 扱いなのでチェック不要
            debug_assert!(edge.cost >= F::zero());
            debug_assert!(edge.upper >= F::zero());

            let (u, v) = (edge.u, edge.v);

            let arc_id_u = ArcId(self.start[u.index()] + counter[u.index()]);
            counter[u.index()] += 1;
            let arc_id_v = ArcId(self.start[v.index()] + counter[v.index()]);
            counter[v.index()] += 1;

            // 元の edge_index -> 正規化後 forward arc（u->v）の inside index
            self.edge_id_to_arc_id[edge.edge_index] = arc_id_u;

            // u -> v
            self.to[arc_id_u.index()] = v;
            self.upper[arc_id_u.index()] = edge.upper;
            self.cost[arc_id_u.index()] = edge.cost;
            self.rev[arc_id_u.index()] = arc_id_v;

            // v -> u (reverse arc)
            self.to[arc_id_v.index()] = u;
            self.flow[arc_id_v.index()] = edge.upper; // あなたの表現に合わせる
            self.upper[arc_id_v.index()] = edge.upper;
            self.cost[arc_id_v.index()] = -edge.cost;
            self.rev[arc_id_v.index()] = arc_id_u;
        }
    }

    pub fn get_flow(
        &self,
        graph: &Graph<Directed, MinimumCostFlowNode<F>, MinimumCostFlowEdge<F>>,
    ) -> Vec<F> {
        // for u in 0..graph.num_nodes() {
        //     graph.excesses[u] = self.excesses[u];
        // }

        let mut flows = Vec::new();
        for edge_id in 0..graph.num_edges() {
            let arc_id = self.edge_id_to_arc_id[edge_id];
            let edge = &graph.get_edge(EdgeId(edge_id)).unwrap();
            // graph.edges[edge_id].data.flow = if edge.data.cost >= F::zero() {
            flows.push(if edge.data.cost >= F::zero() {
                self.flow[arc_id.index()] + edge.data.lower
            } else {
                edge.data.upper - self.flow[arc_id.index()]
            });
            // assert!(graph.edges[edge_id].data.flow <= graph.edges[edge_id].data.upper);
            // assert!(graph.edges[edge_id].data.flow >= graph.edges[edge_id].data.lower);
        }
        flows
    }

    #[inline]
    pub fn neighbors(&self, u: NodeId) -> ArcIdRange {
        ArcIdRange {
            cur: self.start[u.index()],
            end: self.start[u.index() + 1],
        }
    }

    #[inline]
    pub fn push_flow(&mut self, u: NodeId, arc_id: ArcId, flow: F) {
        let rev = self.rev[arc_id.index()];
        let to = self.to[arc_id.index()];
        self.flow[arc_id.index()] += flow;
        self.flow[rev.index()] -= flow;
        self.excesses[u.index()] -= flow;
        self.excesses[to.index()] += flow;
    }

    pub fn calculate_distance_from_source(
        &mut self,
        source: NodeId,
    ) -> (Vec<Option<F>>, Vec<Option<ArcId>>) {
        let mut prev = vec![None; self.num_nodes];
        let mut bh = BinaryHeap::new();
        let mut dist: Vec<Option<F>> = vec![None; self.num_nodes];
        let mut visited = vec![false; self.num_nodes];

        bh.push((Reverse(F::zero()), source));
        dist[source.index()] = Some(F::zero());

        while let Some((d, u)) = bh.pop() {
            if visited[u.index()] {
                continue;
            }
            visited[u.index()] = true;

            for arc_id in self.neighbors(u) {
                if self.residual_capacity(arc_id) == F::zero() {
                    continue;
                }

                let to = self.to[arc_id.index()];
                let new_dist = d.0 + self.reduced_cost(u, arc_id);
                if dist[to.index()].is_none() || dist[to.index()].unwrap() > new_dist {
                    dist[to.index()] = Some(new_dist);
                    prev[to.index()] = Some(arc_id);
                    bh.push((Reverse(new_dist), to));
                }
            }
        }

        (dist, prev)
    }

    // fn calculate_objective_value(&self) -> F {
    //     (0..graph.num_edges()).fold(F::zero(), |cost, edge_id| {
    //         let edge = graph.get_edge(EdgeId(edge_id));
    //         cost + edge.data.cost * flows[edge_id]
    //     })
    // }

    // pub fn minimum_cost(&self) -> F {
    //     let mut c = F::zero();
    //     for i in 0..self.num_edges {
    //         c += self.flow[i] * self.cost[i];
    //     }
    //     c
    // }

    #[inline]
    pub fn reduced_cost(&self, u: NodeId, arc_id: ArcId) -> F {
        self.cost[arc_id.index()] - self.potentials[u.index()]
            + self.potentials[self.to[arc_id.index()].index()]
    }

    #[inline]
    pub fn reduced_cost_rev(&self, u: NodeId, arc_id: ArcId) -> F {
        -(self.cost[arc_id.index()] - self.potentials[u.index()]
            + self.potentials[self.to[arc_id.index()].index()])
    }

    pub fn residual_capacity(&self, arc_id: ArcId) -> F {
        self.upper[arc_id.index()] - self.flow[arc_id.index()]
    }

    pub fn is_feasible(&self, arc_id: ArcId) -> bool {
        F::zero() <= self.flow[arc_id.index()]
            && self.flow[arc_id.index()] <= self.upper[arc_id.index()]
    }
}

pub(crate) fn construct_extend_network_one_supply_one_demand<F>(
    graph: &NormalizedNetwork<'_, F>,
) -> (NodeId, NodeId, Vec<NormalizedEdge<F>>, Vec<F>)
where
    F: CostNum,
{
    let mut edges = Vec::new();
    let mut excess = vec![F::zero(); graph.num_nodes() + 2];
    let source = NodeId(graph.num_nodes());
    let sink = NodeId(source.index() + 1);
    let mut total_excess = F::zero();
    let mut edge_index = graph.num_edges();

    for u in 0..graph.num_nodes() {
        if u == source.index() || u == sink.index() {
            continue;
        }
        if graph.excesses()[u] > F::zero() {
            edges.push(NormalizedEdge {
                u: source,
                v: NodeId(u),
                upper: graph.excesses()[u],
                cost: F::zero(),
                edge_index,
            });
            edge_index += 1;
            total_excess += graph.excesses()[u];
        }
        if graph.excesses()[u] < F::zero() {
            edges.push(NormalizedEdge {
                u: NodeId(u),
                v: sink,
                upper: -graph.excesses()[u],
                cost: F::zero(),
                edge_index,
            });
        }
        excess[u] -= graph.excesses()[u];
    }
    excess[source.index()] = total_excess;
    excess[sink.index()] = -total_excess;

    (source, sink, edges, excess)
}
//
// pub(crate) fn construct_extend_network_feasible_solution<F>(graph: &mut Graph<Directed, MinimumCostFlowNode<F>, MinimumCostFlowEdge<F>>) -> (NodeId, Vec<NodeId>, Vec<EdgeId>)
// where
//     F: MinimumCostFlowNum,
// {
//     let inf_cost = graph.edges.iter().map(|e| e.data.cost).fold(F::one(), |acc, cost| acc + cost); // all edge costs are non-negative
//
//     // add artificial nodes
//     let root = graph.add_node();
//
//     // add artificial edges
//     let mut artificial_edges = Vec::new();
//     for u in 0..graph.num_nodes() {
//         if u == root.index() {
//             continue;
//         }
//
//         let excess = graph.nodes[u].data.b;
//         if excess >= F::zero() {
//             // u -> root
//             let edge_id = graph.add_edge(NodeId(u), root, CapCostEdge{flow: F::zero(), lower: F::zero(), upper: excess, cost: inf_cost});
//             graph.edges[edge_id.index()].data.flow = excess;
//             artificial_edges.push(edge_id);
//         } else {
//             // root -> u
//             let edge_id = graph.add_edge(root, NodeId(u), CapCostEdge{flow: F::zero(), lower: F::zero(), upper: -excess, cost:inf_cost});
//             graph.edges[edge_id.index()].data.flow = -excess;
//             artificial_edges.push(edge_id);
//         }
//         graph.nodes[u].data.b = F::zero();
//     }
//
//     (root, vec![root], artificial_edges)
// }
