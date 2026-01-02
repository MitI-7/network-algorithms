use crate::{
    algorithms::maximum_flow::edge::MaximumFlowEdge,
    core::numeric::FlowNum,
    graph::{
        direction::Directed,
        graph::Graph,
        ids::{ArcId, EdgeId, NodeId},
        iter::ArcIdRange,
    },
};
use std::{collections::VecDeque, marker::PhantomData};

#[derive(Default)]
pub(crate) struct ResidualNetwork<N, F> {
    pub(crate) num_nodes: usize,
    pub(crate) num_edges: usize,
    pub(crate) edge_id_to_arc_id: Box<[ArcId]>,

    // invariant attribute
    pub(crate) start: Box<[usize]>,
    pub(crate) to: Box<[NodeId]>,
    pub(crate) rev: Box<[ArcId]>,

    // state
    pub(crate) residual_capacities: Box<[F]>,
    pub(crate) excesses: Box<[F]>,
    pub(crate) distances_to_sink: Box<[usize]>, // distance from u to sink in residual network
    que: VecDeque<NodeId>,

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
        self.to = vec![NodeId(usize::MAX); self.num_edges * 2].into_boxed_slice();
        self.rev = vec![ArcId(usize::MAX); self.num_edges * 2].into_boxed_slice();
        self.residual_capacities = vec![F::zero(); self.num_edges * 2].into_boxed_slice();
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
            let (u, v) = (e.u, e.v);
            let arc_id_u = ArcId(self.start[u.index()] + counter[u.index()]);
            counter[u.index()] += 1;
            let arc_id_v = ArcId(self.start[v.index()] + counter[v.index()]);
            counter[v.index()] += 1;

            self.edge_id_to_arc_id[edge_index] = arc_id_u;

            // u -> v
            self.to[arc_id_u.index()] = v;
            self.rev[arc_id_u.index()] = arc_id_v;
            self.residual_capacities[arc_id_u.index()] = e.data.upper;

            // v -> u
            self.to[arc_id_v.index()] = u;
            self.rev[arc_id_v.index()] = arc_id_u;
            self.residual_capacities[arc_id_v.index()] = F::zero();
        }
    }

    pub(crate) fn get_flows(&self, graph: &Graph<Directed, N, MaximumFlowEdge<F>>) -> Vec<F> {
        self.edge_id_to_arc_id
            .iter()
            .enumerate()
            .map(|(edge_id, &arc_id)| {
                graph.get_edge(EdgeId(edge_id)).unwrap().data.upper
                    - self.residual_capacities[arc_id.index()]
            })
            .collect()
    }

    #[inline]
    pub(crate) fn neighbors(&self, u: NodeId) -> ArcIdRange {
        ArcIdRange {
            cur: self.start[u.index()],
            end: self.start[u.index() + 1],
        }
    }

    #[inline]
    pub(crate) fn push_flow(&mut self, u: NodeId, arc_id: ArcId, flow: F, without_excess: bool) {
        self.residual_capacities[arc_id.index()] -= flow;
        self.residual_capacities[self.rev[arc_id.index()].index()] += flow;

        if !without_excess {
            self.excesses[u.index()] -= flow;
            self.excesses[self.to[arc_id.index()].index()] += flow;
        }
    }

    // // O(n + m)
    // // calculate the distance from u to sink in the residual network
    // // if such a path does not exist, distance[u] becomes self.num_nodes
    // pub(crate) fn update_distances_to_sink(&mut self, source: NodeId, sink: NodeId) {
    //     self.que.clear();
    //     self.que.push_back(sink);
    //     self.distances_to_sink.fill(self.num_nodes);
    //     self.distances_to_sink[sink.index()] = 0;
    //
    //     while let Some(v) = self.que.pop_front() {
    //         for arc_id in self.neighbors(v) {
    //             // e.to -> v
    //             let to = self.to[arc_id.index()];
    //             if self.flow[arc_id.index()] > F::zero()
    //                 && self.distances_to_sink[to.index()] == self.num_nodes
    //             {
    //                 self.distances_to_sink[to.index()] = self.distances_to_sink[v.index()] + 1;
    //                 if to != source {
    //                     self.que.push_back(to);
    //                 }
    //             }
    //         }
    //     }
    // }
    //
    // #[inline]
    // pub(crate) fn is_admissible_edge(&self, from: NodeId, arc_id: ArcId) -> bool {
    //     self.residual_capacity(arc_id) > F::zero()
    //         && self.distances_to_sink[from.index()]
    //             == self.distances_to_sink[self.to[arc_id.index()].index()] + 1
    // }

    pub(crate) fn residual_capacity(&self, arc_id: ArcId) -> F {
        self.residual_capacities[arc_id.index()]
    }
}
