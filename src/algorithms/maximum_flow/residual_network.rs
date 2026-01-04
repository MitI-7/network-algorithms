use crate::direction::Direction;
use crate::{
    algorithms::maximum_flow::edge::MaximumFlowEdge,
    core::numeric::FlowNum,
    graph::{
        graph::Graph,
        ids::{ArcId, NodeId},
        iter::ArcIdRange,
    },
};
use std::collections::VecDeque;

#[derive(Default)]
pub(crate) struct ResidualNetwork<F> {
    pub(crate) num_nodes: usize,
    pub(crate) num_edges: usize,
    pub(crate) edge_id_to_arc_id: Box<[ArcId]>,

    pub(crate) start: Box<[usize]>,
    pub(crate) upper: Box<[F]>,
    pub(crate) to: Box<[NodeId]>,
    pub(crate) rev: Box<[ArcId]>,

    pub(crate) residual_capacities: Box<[F]>,
    pub(crate) excesses: Box<[F]>,
    pub(crate) distances_to_sink: Box<[usize]>,
    que: VecDeque<NodeId>,
}

impl<F> ResidualNetwork<F>
where
    F: FlowNum,
{
    pub fn new<D: Direction, N>(graph: &Graph<D, N, MaximumFlowEdge<F>>) -> Self {
        let mut rn = Self {
            num_nodes: graph.num_nodes(),
            num_edges: graph.num_edges(),
            edge_id_to_arc_id: vec![ArcId(usize::MAX); graph.num_edges()].into_boxed_slice(),
            start: vec![0; graph.num_nodes() + 1].into_boxed_slice(),
            upper: vec![F::zero(); graph.num_edges() * 2].into_boxed_slice(),
            to: vec![NodeId(usize::MAX); graph.num_edges() * 2].into_boxed_slice(),
            rev: vec![ArcId(usize::MAX); graph.num_edges() * 2].into_boxed_slice(),
            residual_capacities: vec![F::zero(); graph.num_edges() * 2].into_boxed_slice(),
            excesses: vec![F::zero(); graph.num_nodes()].into_boxed_slice(),
            distances_to_sink: vec![0; graph.num_nodes()].into_boxed_slice(),
            que: VecDeque::new(),
        };
        rn.build(graph);

        rn
    }

    fn build<D: Direction, N>(&mut self, graph: &Graph<D, N, MaximumFlowEdge<F>>) {
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
            let (u, v) = (e.u, e.v);
            let arc_id_u = ArcId(self.start[u.index()] + counter[u.index()]);
            counter[u.index()] += 1;
            let arc_id_v = ArcId(self.start[v.index()] + counter[v.index()]);
            counter[v.index()] += 1;

            self.edge_id_to_arc_id[edge_index] = arc_id_u;

            let upper = e.data.upper;
            let rev_init = if D::IS_DIRECTED { F::zero() } else { upper };

            // u -> v
            self.to[arc_id_u.index()] = v;
            self.rev[arc_id_u.index()] = arc_id_v;
            self.upper[arc_id_u.index()] = upper;
            self.residual_capacities[arc_id_u.index()] = upper;

            // v -> u
            self.to[arc_id_v.index()] = u;
            self.rev[arc_id_v.index()] = arc_id_u;
            self.upper[arc_id_v.index()] = rev_init;
            self.residual_capacities[arc_id_v.index()] = rev_init;
        }
    }

    pub(crate) fn get_flows(&self, residual_capacities: &[F]) -> Vec<F> {
        self.edge_id_to_arc_id
            .iter()
            .map(|&arc_id| self.upper[arc_id.index()] - residual_capacities[arc_id.index()])
            .collect()
    }

    #[inline]
    pub(crate) fn neighbors(&self, u: NodeId) -> ArcIdRange {
        ArcIdRange { cur: self.start[u.index()], end: self.start[u.index() + 1] }
    }

    #[inline]
    pub(crate) fn push_flow(&mut self, u: NodeId, arc_id: ArcId, flow: F) {
        self.push_flow_without_excess(u, arc_id, flow);
        self.excesses[u.index()] -= flow;
        self.excesses[self.to[arc_id.index()].index()] += flow;
    }

    #[inline]
    pub(crate) fn push_flow_without_excess(&mut self, u: NodeId, arc_id: ArcId, flow: F) {
        self.residual_capacities[arc_id.index()] -= flow;
        self.residual_capacities[self.rev[arc_id.index()].index()] += flow;
    }

    pub(crate) fn update_distances_to_sink(&mut self, source: NodeId, sink: NodeId) {
        self.que.clear();
        self.que.push_back(sink);
        self.distances_to_sink.fill(self.num_nodes);
        self.distances_to_sink[sink.index()] = 0;

        while let Some(v) = self.que.pop_front() {
            for arc_id in self.neighbors(v) {
                let to = self.to[arc_id.index()];
                let rev_arc_id = self.rev[arc_id.index()];
                if self.residual_capacities[rev_arc_id.index()] > F::zero()
                    && self.distances_to_sink[to.index()] == self.num_nodes
                {
                    self.distances_to_sink[to.index()] = self.distances_to_sink[v.index()] + 1;
                    if to != source {
                        self.que.push_back(to);
                    }
                }
            }
        }
    }

    #[inline]
    pub(crate) fn is_admissible_arc(&self, from: NodeId, arc_id: ArcId) -> bool {
        self.residual_capacities[arc_id.index()] > F::zero()
            && self.distances_to_sink[from.index()] == self.distances_to_sink[self.to[arc_id.index()].index()] + 1
    }
}
