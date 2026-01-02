use crate::{
    algorithms::maximum_flow::edge::MaximumFlowEdge,
    core::numeric::FlowNum,
    graph::{direction::Directed, graph::Graph, ids::ArcId, iter::ArcIdRange},
};
use std::collections::VecDeque;
use std::marker::PhantomData;

#[derive(Default)]
pub(crate) struct ResidualNetwork<N, F> {
    pub(crate) num_nodes: usize,
    pub(crate) num_edges: usize,
    pub(crate) edge_id_to_arc_id: Box<[ArcId]>,

    pub(crate) start: Box<[usize]>,
    pub(crate) to: Box<[usize]>,
    pub(crate) flow: Box<[F]>,
    pub(crate) upper: Box<[F]>,
    pub(crate) rev: Box<[usize]>,
    pub(crate) excesses: Box<[F]>,

    pub(crate) distances_to_sink: Box<[usize]>, // distance from u to sink in residual network
    que: VecDeque<usize>,

    phantom_data: PhantomData<N>,
}

impl<N, F> ResidualNetwork<N, F>
where
    F: FlowNum,
{
    pub fn build(&mut self, graph: &Graph<Directed, N, MaximumFlowEdge<F>>) {
        self.num_nodes = graph.num_nodes();
        self.num_edges = graph.num_edges();

        // initialize
        self.edge_id_to_arc_id = vec![ArcId(usize::MAX); self.num_edges].into_boxed_slice();
        self.start = vec![0; self.num_nodes + 1].into_boxed_slice();
        self.to = vec![usize::MAX; self.num_edges * 2].into_boxed_slice();
        self.flow = vec![F::zero(); self.num_edges * 2].into_boxed_slice();
        self.upper = vec![F::zero(); self.num_edges * 2].into_boxed_slice();
        self.rev = vec![usize::MAX; self.num_edges * 2].into_boxed_slice();
        self.excesses = vec![F::zero(); self.num_nodes].into_boxed_slice();
        self.distances_to_sink = vec![self.num_nodes; self.num_nodes].into_boxed_slice();

        let mut degree = vec![0; self.num_nodes].into_boxed_slice();

        for edge in graph.edges() {
            degree[edge.u.index()] += 1;
            degree[edge.v.index()] += 1;
        }

        for u in 1..=self.num_nodes {
            self.start[u] += self.start[u - 1] + degree[u - 1];
        }

        let mut counter = vec![0; self.num_nodes];
        for (edge_index, e) in graph.edges().enumerate() {
            let (u, v) = (e.u.index(), e.v.index());
            let arc_id_u = self.start[u] + counter[u];
            counter[u] += 1;
            let arc_id_v = self.start[v] + counter[v];
            counter[v] += 1;

            self.edge_id_to_arc_id[edge_index] = ArcId(arc_id_u);

            // u -> v
            self.to[arc_id_u] = v;
            self.flow[arc_id_u] = F::zero();
            self.upper[arc_id_u] = e.data.upper;
            self.rev[arc_id_u] = arc_id_v;

            // v -> u
            self.to[arc_id_v] = u;
            self.flow[arc_id_v] = e.data.upper;
            self.upper[arc_id_v] = e.data.upper;
            self.rev[arc_id_v] = arc_id_u;
        }
    }

    pub fn get_flows(&self) -> Vec<F> {
        self.edge_id_to_arc_id
            .iter()
            .map(|&arc_id| self.flow[arc_id.index()])
            .collect()
    }

    // #[inline]
    // pub fn neighbors(&self, u: usize) -> impl Iterator<Item = ArcId> + '_ {
    //     (self.start[u]..self.start[u + 1]).map(ArcId)
    // }

    #[inline]
    pub fn neighbors(&self, u: usize) -> ArcIdRange {
        ArcIdRange {
            cur: self.start[u],
            end: self.start[u + 1],
        }
    }

    #[inline]
    pub fn push_flow(&mut self, u: usize, arc_id: ArcId, flow: F, without_excess: bool) {
        self.flow[arc_id.index()] += flow;
        self.flow[self.rev[arc_id.index()]] -= flow;

        if !without_excess {
            self.excesses[u] -= flow;
            self.excesses[self.to[arc_id.index()]] += flow;
        }
    }

    // O(n + m)
    // calculate the distance from u to sink in the residual network
    // if such a path does not exist, distance[u] becomes self.num_nodes
    pub fn update_distances_to_sink(&mut self, source: usize, sink: usize) {
        self.que.clear();
        self.que.push_back(sink);
        self.distances_to_sink.fill(self.num_nodes);
        self.distances_to_sink[sink] = 0;

        while let Some(v) = self.que.pop_front() {
            for arc_id in self.neighbors(v) {
                // e.to -> v
                let to = self.to[arc_id.index()];
                if self.flow[arc_id.index()] > F::zero()
                    && self.distances_to_sink[to] == self.num_nodes
                {
                    self.distances_to_sink[to] = self.distances_to_sink[v] + 1;
                    if to != source {
                        self.que.push_back(to);
                    }
                }
            }
        }
    }

    #[inline]
    pub fn is_admissible_edge(&self, from: usize, arc_id: ArcId) -> bool {
        self.residual_capacity(arc_id) > F::zero()
            && self.distances_to_sink[from] == self.distances_to_sink[self.to[arc_id.index()]] + 1
    }

    pub fn residual_capacity(&self, arc_id: ArcId) -> F {
        self.upper[arc_id.index()] - self.flow[arc_id.index()]
    }
}
