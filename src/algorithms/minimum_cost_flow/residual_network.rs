use crate::algorithms::minimum_cost_flow::error::MinimumCostFlowError;
use crate::{
    algorithms::minimum_cost_flow::normalized_network::{NormalizedEdge, NormalizedNetwork},
    core::numeric::CostNum,
    graph::{
        ids::{ArcId, EdgeId, INVALID_ARC_ID, INVALID_NODE_ID, NodeId},
        iter::ArcIdRange,
    },
};
use std::{cmp::Reverse, collections::BinaryHeap};

#[derive(Default)]
pub(crate) struct ResidualNetwork<F> {
    pub(crate) num_nodes: usize,
    pub(crate) num_edges: usize,
    pub(crate) edge_id_to_arc_id: Box<[ArcId]>,

    pub(crate) start: Box<[usize]>,
    pub(crate) to: Box<[NodeId]>,
    pub(crate) upper: Box<[F]>,
    pub(crate) cost: Box<[F]>,
    pub(crate) rev: Box<[ArcId]>,

    // state
    pub(crate) residual_capacity: Box<[F]>,
    pub(crate) excesses: Box<[F]>,
    pub(crate) potentials: Box<[F]>,

    // ex
    pub(crate) num_nodes_original_graph: usize,
    pub(crate) num_edges_original_graph: usize,
    pub(crate) b: Box<[F]>,
    pub(crate) is_reversed_in_original_graph: Box<[bool]>,
    pub(crate) lower_in_original_graph: Box<[F]>,
}

impl<F> ResidualNetwork<F>
where
    F: CostNum,
{
    pub fn new(
        graph: &NormalizedNetwork<'_, F>,
        artificial_nodes: Option<&[NodeId]>,
        artificial_edges: Option<&[NormalizedEdge<F>]>,
        initial_flows: Option<&[F]>,
        fix_excesses: Option<&[F]>,
    ) -> Self {
        let num_nodes = graph.num_nodes() + artificial_nodes.unwrap_or(&[]).len();
        let num_edges = graph.num_edges() + artificial_edges.unwrap_or(&[]).len();

        let mut rn = Self {
            num_nodes,
            num_edges,
            edge_id_to_arc_id: vec![INVALID_ARC_ID; num_edges].into_boxed_slice(),

            start: vec![0; num_nodes + 1].into_boxed_slice(),
            to: vec![INVALID_NODE_ID; num_edges * 2].into_boxed_slice(),
            upper: vec![F::zero(); num_edges * 2].into_boxed_slice(),
            cost: vec![F::zero(); num_edges * 2].into_boxed_slice(),
            rev: vec![INVALID_ARC_ID; num_edges * 2].into_boxed_slice(),

            residual_capacity: vec![F::zero(); num_edges * 2].into_boxed_slice(),
            excesses: vec![F::zero(); num_nodes].into_boxed_slice(),
            potentials: vec![F::zero(); num_nodes].into_boxed_slice(),

            num_nodes_original_graph: graph.num_nodes(),
            num_edges_original_graph: graph.num_edges(),
            b: vec![F::zero(); num_nodes].into_boxed_slice(),
            is_reversed_in_original_graph: vec![false; num_edges].into_boxed_slice(),
            lower_in_original_graph: vec![F::zero(); num_edges].into_boxed_slice(),
        };
        rn.build(graph, artificial_nodes, artificial_edges, initial_flows, fix_excesses);

        rn
    }
    fn build(
        &mut self,
        graph: &NormalizedNetwork<'_, F>,
        _artificial_nodes: Option<&[NodeId]>,
        artificial_edges: Option<&[NormalizedEdge<F>]>,
        initial_flows: Option<&[F]>,
        fix_excesses: Option<&[F]>,
    ) {
        if graph.num_nodes() == 0 {
            return;
        }

        for (u, e) in graph.excesses().iter().enumerate() {
            self.excesses[u] = *e;
        }

        if let Some(fix) = fix_excesses {
            for u in 0..self.num_nodes {
                self.excesses[u] += fix[u];
            }
        }
        self.b = self.excesses.clone();

        let mut degree = vec![0usize; self.num_nodes];

        for ne in graph
            .iter_edges()
            .chain(artificial_edges.into_iter().flatten().copied())
        {
            degree[ne.u.index()] += 1;
            degree[ne.v.index()] += 1;
        }

        for u in 1..=self.num_nodes {
            self.start[u] = self.start[u - 1] + degree[u - 1];
        }

        let mut counter = vec![0usize; self.num_nodes];

        for (edge_id, edge) in graph
            .iter_edges()
            .chain(artificial_edges.into_iter().flatten().copied())
            .enumerate()
        {
            debug_assert!(edge.cost >= F::zero());
            debug_assert!(edge.upper >= F::zero());

            let (u, v) = (edge.u, edge.v);

            let arc_id_u = ArcId(self.start[u.index()] + counter[u.index()]);
            counter[u.index()] += 1;
            let arc_id_v = ArcId(self.start[v.index()] + counter[v.index()]);
            counter[v.index()] += 1;

            self.edge_id_to_arc_id[edge_id] = arc_id_u;
            self.lower_in_original_graph[edge_id] = edge.lower;
            self.is_reversed_in_original_graph[edge_id] = edge.is_reversed;

            let initial_flow = initial_flows.map_or(F::zero(), |init| init[edge_id]);
            // u -> v
            self.to[arc_id_u.index()] = v;
            self.upper[arc_id_u.index()] = edge.upper;
            self.cost[arc_id_u.index()] = edge.cost;
            self.rev[arc_id_u.index()] = arc_id_v;
            self.residual_capacity[arc_id_u.index()] = edge.upper - initial_flow;

            // v -> u (reverse arc)
            self.to[arc_id_v.index()] = u;
            self.upper[arc_id_v.index()] = edge.upper;
            self.cost[arc_id_v.index()] = -edge.cost;
            self.rev[arc_id_v.index()] = arc_id_u;
            self.residual_capacity[arc_id_v.index()] = initial_flow;
        }
    }

    #[inline]
    pub fn neighbors(&self, u: NodeId) -> ArcIdRange {
        ArcIdRange { cur: self.start[u.index()], end: self.start[u.index() + 1] }
    }

    #[inline]
    pub fn push_flow(&mut self, u: NodeId, arc_id: ArcId, flow: F) {
        let rev = self.rev[arc_id.index()];
        let to = self.to[arc_id.index()];
        self.residual_capacity[arc_id.index()] -= flow;
        self.residual_capacity[rev.index()] += flow;
        self.excesses[u.index()] -= flow;
        self.excesses[to.index()] += flow;
    }

    pub fn calculate_distance_from_source(&mut self, source: NodeId) -> (Vec<Option<F>>, Vec<Option<ArcId>>) {
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

    #[inline]
    pub fn reduced_cost(&self, u: NodeId, arc_id: ArcId) -> F {
        self.cost[arc_id.index()] - self.potentials[u.index()] + self.potentials[self.to[arc_id.index()].index()]
    }

    #[inline]
    pub fn reduced_cost_rev(&self, u: NodeId, arc_id: ArcId) -> F {
        -(self.cost[arc_id.index()] - self.potentials[u.index()] + self.potentials[self.to[arc_id.index()].index()])
    }

    pub fn residual_capacity(&self, arc_id: ArcId) -> F {
        self.residual_capacity[arc_id.index()]
    }

    pub fn have_flow_in_artificial_arc(&self) -> bool {
        (self.num_edges_original_graph..self.num_edges)
            .into_iter()
            .any(|edge_id| {
                let arc_id = self.edge_id_to_arc_id[edge_id];
                self.residual_capacity[arc_id.index()] != self.upper[arc_id.index()]
            })
    }

    pub fn have_excess(&self) -> bool {
        self.excesses.iter().any(|e| *e != F::zero())
    }

    pub fn calculate_objective_value_original_graph(&self) -> F {
        let mut objective_value = F::zero();
        for edge_id in 0..self.num_edges_original_graph {
            let arc_id = self.edge_id_to_arc_id[edge_id];
            let cost = if self.is_reversed_in_original_graph[edge_id] {
                -self.cost[arc_id.index()]
            } else {
                self.cost[arc_id.index()]
            };
            objective_value += cost * self.flow_original_graph(EdgeId(edge_id));
        }
        objective_value
    }

    pub(crate) fn flow_original_graph(&self, edge_id: EdgeId) -> F {
        let arc_id = self.edge_id_to_arc_id[edge_id.index()];
        let flow = self.upper[arc_id.index()] - self.residual_capacity[arc_id.index()];

        if self.is_reversed_in_original_graph[edge_id.index()] {
            self.upper[arc_id.index()] + self.lower_in_original_graph[edge_id.index()] - flow
        } else {
            flow + self.lower_in_original_graph[edge_id.index()]
        }
    }

    pub(crate) fn flows_original_graph(&self) -> Vec<F> {
        (0..self.num_edges_original_graph)
            .map(|edge_id| self.flow_original_graph(EdgeId(edge_id)))
            .collect()
    }

    pub(crate) fn potential_original_graph(&self, node_id: NodeId) -> F {
        // if node_id.index() >= self.num_nodes_original_graph {
        //     return None;
        // }
        self.potentials[node_id.index()]
    }

    pub(crate) fn potentials_original_graph(&self) -> Vec<F> {
        self.potentials[..self.num_nodes_original_graph].to_vec()
    }

    pub(crate) fn check_optimality(&self) -> bool {
        let mut ok = true;
        for u in (0..self.num_nodes).map(NodeId) {
            for arc_id in self.neighbors(u) {
                if self.upper[arc_id.index()] == F::zero() {
                    continue;
                }

                let f = self.upper[arc_id.index()] - self.residual_capacity[arc_id.index()];
                let r = self.reduced_cost(u, arc_id);

                // Complementary slackness (optimality witness by potentials)
                ok &= if f == F::zero() {
                    r >= F::zero()
                } else if F::zero() < f && f < self.upper[arc_id.index()] {
                    r == F::zero()
                } else {
                    r <= F::zero()
                };
            }
        }
        ok
    }
}
