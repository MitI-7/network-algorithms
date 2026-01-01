use crate::algorithms::minimum_cost_flow::normalized_network::NormalizedNetwork;
use crate::{
    algorithms::minimum_cost_flow::{
        MinimumCostFlowNum, edge::MinimumCostFlowEdge, node::MinimumCostFlowNode,
    },
    graph::{
        direction::Directed,
        graph::Graph,
        ids::{EdgeId, NodeId},
    },
};
use std::cmp::Reverse;
use std::collections::BinaryHeap;

#[derive(Default)]
pub struct ResidualNetwork<F> {
    pub num_nodes: usize,
    pub num_edges: usize,
    pub edge_index_to_inside_edge_index: Box<[usize]>,

    pub excesses: Box<[F]>,
    pub potentials: Box<[F]>,

    pub start: Box<[usize]>,
    pub to: Box<[usize]>,
    pub flow: Box<[F]>,
    pub upper: Box<[F]>,
    pub cost: Box<[F]>,
    pub rev: Box<[usize]>,

    pub is_reversed: Box<[bool]>,
}

impl<F> ResidualNetwork<F>
where
    F: MinimumCostFlowNum,
{
    pub fn build(
        &mut self,
        graph: &NormalizedNetwork<'_, F>,
        artificial_nodes: Option<&[NodeId]>,
        artificial_edges: Option<&[MinimumCostFlowEdge<F>]>,
    ) {
        if graph.num_nodes() == 0 {
            return;
        }

        self.num_nodes = graph.num_nodes(); // + artificial_nodes.unwrap_or(&[]).len();
        self.num_edges = graph.num_edges(); // + artificial_edges.unwrap_or(&[]).len();

        // b は正規化後のものを使う
        self.excesses = graph.excesses().to_vec().into_boxed_slice();

        self.edge_index_to_inside_edge_index = vec![usize::MAX; self.num_edges].into_boxed_slice();
        self.start = vec![0; self.num_nodes + 1].into_boxed_slice();
        self.to = vec![usize::MAX; self.num_edges * 2].into_boxed_slice();
        self.flow = vec![F::zero(); self.num_edges * 2].into_boxed_slice();
        self.upper = vec![F::zero(); self.num_edges * 2].into_boxed_slice();
        self.cost = vec![F::zero(); self.num_edges * 2].into_boxed_slice();
        self.rev = vec![usize::MAX; self.num_edges * 2].into_boxed_slice();
        self.potentials = vec![F::zero(); self.num_nodes].into_boxed_slice();

        self.make_csr(graph, artificial_nodes, artificial_edges);
    }

    fn make_csr(
        &mut self,
        graph: &NormalizedNetwork<'_, F>,
        _artificial_nodes: Option<&[NodeId]>,
        _artificial_edges: Option<&[MinimumCostFlowEdge<F>]>,
    ) {
        let mut degree = vec![0usize; self.num_nodes];

        // 正規化済みの (u->v) を使って次数を数える
        for ne in graph.iter_edges() {
            degree[ne.u.index()] += 1;
            degree[ne.v.index()] += 1;
        }

        // start の prefix sum
        for i in 1..=self.num_nodes {
            self.start[i] = self.start[i - 1] + degree[i - 1];
        }

        let mut counter = vec![0usize; self.num_nodes];

        for edge in graph.iter_edges() {
            // ここでは lower は常に 0 扱いなのでチェック不要
            debug_assert!(edge.cost >= F::zero());
            debug_assert!(edge.upper >= F::zero());

            let u = edge.u.index();
            let v = edge.v.index();

            let inside_edge_index_u = self.start[u] + counter[u];
            counter[u] += 1;
            let inside_edge_index_v = self.start[v] + counter[v];
            counter[v] += 1;

            // 元の edge_index -> 正規化後 forward arc（u->v）の inside index
            self.edge_index_to_inside_edge_index[edge.edge_index] = inside_edge_index_u;

            // u -> v
            self.to[inside_edge_index_u] = v;
            self.upper[inside_edge_index_u] = edge.upper;
            self.cost[inside_edge_index_u] = edge.cost;
            self.rev[inside_edge_index_u] = inside_edge_index_v;

            // v -> u (reverse arc)
            self.to[inside_edge_index_v] = u;
            self.flow[inside_edge_index_v] = edge.upper; // あなたの表現に合わせる
            self.upper[inside_edge_index_v] = edge.upper;
            self.cost[inside_edge_index_v] = -edge.cost;
            self.rev[inside_edge_index_v] = inside_edge_index_u;
        }
    }

    pub fn set_flow(
        &self,
        graph: &mut Graph<Directed, MinimumCostFlowNode<F>, MinimumCostFlowEdge<F>>,
    ) -> Vec<F> {
        // for u in 0..graph.num_nodes() {
        //     graph.excesses[u] = self.excesses[u];
        // }

        let mut flows = Vec::new();
        for edge_id in 0..graph.num_edges() {
            let i = self.edge_index_to_inside_edge_index[edge_id];
            let edge = &graph.edges[edge_id];
            // graph.edges[edge_id].data.flow = if edge.data.cost >= F::zero() {
            flows.push(if edge.data.cost >= F::zero() {
                self.flow[i] + edge.data.lower
            } else {
                edge.data.upper - self.flow[i]
            });
            // assert!(graph.edges[edge_id].data.flow <= graph.edges[edge_id].data.upper);
            // assert!(graph.edges[edge_id].data.flow >= graph.edges[edge_id].data.lower);
        }
        flows
    }

    #[inline]
    pub fn neighbors(&self, u: usize) -> std::ops::Range<usize> {
        self.start[u]..self.start[u + 1]
    }

    #[inline]
    pub fn push_flow(&mut self, u: usize, i: usize, flow: F) {
        let rev = self.rev[i];
        let to = self.to[i];
        self.flow[i] += flow;
        self.flow[rev] -= flow;
        self.excesses[u] -= flow;
        self.excesses[to] += flow;
    }

    pub fn calculate_distance_from_source(
        &mut self,
        source: usize,
    ) -> (Vec<Option<F>>, Vec<Option<usize>>) {
        let mut prev = vec![None; self.num_nodes];
        let mut bh = BinaryHeap::new();
        let mut dist: Vec<Option<F>> = vec![None; self.num_nodes];
        let mut visited = vec![false; self.num_nodes];

        bh.push((Reverse(F::zero()), source));
        dist[source] = Some(F::zero());

        while let Some((d, u)) = bh.pop() {
            if visited[u] {
                continue;
            }
            visited[u] = true;

            for edge_id in self.neighbors(u) {
                if self.residual_capacity(edge_id) == F::zero() {
                    continue;
                }

                let to = self.to[edge_id];
                let new_dist = d.0 + self.reduced_cost(u, edge_id);
                if dist[to].is_none() || dist[to].unwrap() > new_dist {
                    dist[to] = Some(new_dist);
                    prev[to] = Some(edge_id);
                    bh.push((Reverse(new_dist), to));
                }
            }
        }

        (dist, prev)
    }

    pub fn minimum_cost(&self) -> F {
        let mut c = F::zero();
        for i in 0..self.num_edges {
            c += self.flow[i] * self.cost[i];
        }
        c
    }

    #[inline]
    pub fn reduced_cost(&self, u: usize, i: usize) -> F {
        self.cost[i] - self.potentials[u] + self.potentials[self.to[i]]
    }

    #[inline]
    pub fn reduced_cost_rev(&self, u: usize, i: usize) -> F {
        -(self.cost[i] - self.potentials[u] + self.potentials[self.to[i]])
    }

    pub fn residual_capacity(&self, i: usize) -> F {
        self.upper[i] - self.flow[i]
    }

    pub fn is_feasible(&self, i: usize) -> bool {
        F::zero() <= self.flow[i] && self.flow[i] <= self.upper[i]
    }
}
//
// pub(crate) fn construct_extend_network_one_supply_one_demand<F>(
//     graph: &mut Graph<Directed, MinimumCostFlowNode<F>, MinimumCostFlowEdge<F>>,
// ) -> (NodeId, NodeId, Vec<MinimumCostFlowEdge<F>>)
// where
//     F: MinimumCostFlowNum,
// {
//     let artificial_edges = Vec::new();
//     let source = graph.add_node();
//     let sink = graph.add_node();
//     let mut total_excess = F::zero();
//
//     for u in 0..graph.num_nodes() {
//         if u == source.index() || u == sink.index() {
//             continue;
//         }
//         if graph.nodes[u].data.b > F::zero() {
//             graph.add_edge(source, NodeId(u), MinimumCostFlowEdge { flow: F::zero(), lower: F::zero(), upper: graph.nodes[u].data.b, cost: F::zero() });
//             total_excess += graph.nodes[u].data.b;
//         }
//         if graph.nodes[u].data.b < F::zero() {
//             graph.add_edge(NodeId(u), sink, MinimumCostFlowEdge { flow: F::zero(), lower: F::zero(), upper: -graph.nodes[u].data.b, cost: F::zero() });
//         }
//         graph.nodes[u].data.b = F::zero();
//     }
//     graph.nodes[source.index()].data.b = total_excess;
//     graph.nodes[sink.index()].data.b = -total_excess;
//
//     (source, sink, artificial_edges)
// }
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
