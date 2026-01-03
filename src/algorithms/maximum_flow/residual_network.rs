use crate::{
    algorithms::maximum_flow::edge::MaximumFlowEdge,
    core::numeric::FlowNum,
    graph::{
        direction::Directed,
        graph::Graph,
        ids::{ArcId, NodeId},
        iter::ArcIdRange,
    },
};
use std::marker::PhantomData;

#[derive(Default)]
pub(crate) struct ResidualNetwork<N, F> {
    pub(crate) num_nodes: usize,
    pub(crate) num_edges: usize,
    pub(crate) edge_id_to_arc_id: Box<[ArcId]>,

    pub(crate) start: Box<[usize]>,
    pub(crate) upper: Box<[F]>,
    pub(crate) to: Box<[NodeId]>,
    pub(crate) rev: Box<[ArcId]>,

    pub(crate) residual_capacities: Box<[F]>,
    phantom_data: PhantomData<N>,
}

impl<N, F> ResidualNetwork<N, F>
where
    F: FlowNum,
{
    pub fn new(graph: &Graph<Directed, N, MaximumFlowEdge<F>>) -> Self {
        let mut rn = Self {
            num_nodes: graph.num_nodes(),
            num_edges: graph.num_edges(),
            edge_id_to_arc_id: vec![ArcId(usize::MAX); graph.num_edges()].into_boxed_slice(),
            start: vec![0; graph.num_nodes() + 1].into_boxed_slice(),
            upper: vec![F::zero(); graph.num_edges() * 2].into_boxed_slice(),
            to: vec![NodeId(usize::MAX); graph.num_edges() * 2].into_boxed_slice(),
            rev: vec![ArcId(usize::MAX); graph.num_edges() * 2].into_boxed_slice(),
            residual_capacities: vec![F::zero(); graph.num_edges() * 2].into_boxed_slice(),
            phantom_data: PhantomData,
        };
        rn.build(graph);

        rn
    }

    fn build(&mut self, graph: &Graph<Directed, N, MaximumFlowEdge<F>>) {
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

            // u -> v
            self.to[arc_id_u.index()] = v;
            self.rev[arc_id_u.index()] = arc_id_v;
            self.upper[arc_id_u.index()] = e.data.upper;

            // v -> u
            self.to[arc_id_v.index()] = u;
            self.rev[arc_id_v.index()] = arc_id_u;
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
    pub(crate) fn push_flow(&mut self, _u: NodeId, arc_id: ArcId, flow: F, excesses: Option<&mut [F]>) {
        self.residual_capacities[arc_id.index()] -= flow;
        self.residual_capacities[self.rev[arc_id.index()].index()] += flow;

        if excesses.is_some() {
            // excesses.unwrap()[u.index()] -= flow;
            // excesses.unwrap()[self.to[arc_id.index()].index()] += flow;
        }
    }
    //
    // #[inline]
    // pub(crate) fn is_admissible_edge(&self, from: NodeId, arc_id: ArcId) -> bool {
    //     self.residual_capacity(arc_id) > F::zero()
    //         && self.distances_to_sink[from.index()]
    //             == self.distances_to_sink[self.to[arc_id.index()].index()] + 1
    // }
}
