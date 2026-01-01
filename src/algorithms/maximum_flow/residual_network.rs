use crate::graph::graph::Graph;
use crate::graph::direction::Directed;
use crate::algorithms::maximum_flow::edge::MaximumFlowEdge;
use std::collections::VecDeque;
use std::marker::PhantomData;
use crate::core::numeric::FlowNum;

#[derive(Default)]
pub struct ResidualNetwork<N, F> {
    pub num_nodes: usize,
    pub num_edges: usize,
    pub edge_index_to_inside_arc_index: Box<[usize]>,

    pub start: Box<[usize]>,
    pub to: Box<[usize]>,
    pub flow: Box<[F]>,
    pub upper: Box<[F]>,
    pub rev: Box<[usize]>,
    pub excesses: Box<[F]>,

    pub distances_to_sink: Box<[usize]>, // distance from u to sink in residual network
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
        self.edge_index_to_inside_arc_index = vec![usize::MAX; self.num_edges].into_boxed_slice();
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
                let inside_edge_index_u = self.start[u] + counter[u];
                counter[u] += 1;
                let inside_edge_index_v = self.start[v] + counter[v];
                counter[v] += 1;

                self.edge_index_to_inside_arc_index[edge_index] = inside_edge_index_u;

                // u -> v
                self.to[inside_edge_index_u] = v;
                self.flow[inside_edge_index_u] = F::zero();
                self.upper[inside_edge_index_u] = e.data.upper;
                self.rev[inside_edge_index_u] = inside_edge_index_v;

                // v -> u
                self.to[inside_edge_index_v] = u;
                self.flow[inside_edge_index_v] = e.data.upper;
                self.upper[inside_edge_index_v] = e.data.upper;
                self.rev[inside_edge_index_v] = inside_edge_index_u;

        }
    }

    pub fn get_flows(&self) -> Vec<F>{
        self
            .edge_index_to_inside_arc_index
            .iter()
            .map(|&inside_edge_id| self.flow[inside_edge_id])
            .collect()
    }

    #[inline]
    pub fn neighbors(&self, u: usize) -> std::ops::Range<usize> {
        self.start[u]..self.start[u + 1]
    }

    #[inline]
    pub fn push_flow(&mut self, u: usize, i: usize, flow: F, without_excess: bool) {
        self.flow[i] += flow;
        self.flow[self.rev[i]] -= flow;

        if !without_excess {
            self.excesses[u] -= flow;
            self.excesses[self.to[i]] += flow;
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
            for i in self.neighbors(v) {
                // e.to -> v
                let to = self.to[i];
                if self.flow[i] > F::zero() && self.distances_to_sink[to] == self.num_nodes {
                    self.distances_to_sink[to] = self.distances_to_sink[v] + 1;
                    if to != source {
                        self.que.push_back(to);
                    }
                }
            }
        }
    }

    #[inline]
    pub fn is_admissible_edge(&self, from: usize, i: usize) -> bool {
        self.residual_capacity(i) > F::zero() && self.distances_to_sink[from] == self.distances_to_sink[self.to[i]] + 1
    }

    pub fn residual_capacity(&self, i: usize) -> F {
        self.upper[i] - self.flow[i]
    }
}
