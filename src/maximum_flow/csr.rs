use crate::maximum_flow::graph::Graph;
use num_traits::NumAssign;
use std::collections::VecDeque;

#[derive(Default)]
pub struct CSR<Flow> {
    pub num_nodes: usize,
    pub num_edges: usize,
    pub edge_index_to_inside_edge_index: Box<[usize]>,

    pub start: Box<[usize]>,
    pub to: Box<[usize]>,
    pub flow: Box<[Flow]>,
    pub upper: Box<[Flow]>,
    pub rev: Box<[usize]>,

    pub distances: Box<[usize]>, // distance from u to sink in residual network
    que: VecDeque<usize>,
}

impl<Flow> CSR<Flow>
where
    Flow: NumAssign + Ord + Copy,
{
    pub fn build(&mut self, graph: &mut Graph<Flow>) {
        self.num_nodes = graph.num_nodes();
        self.num_edges = graph.num_edges();

        // initialize
        self.edge_index_to_inside_edge_index = vec![usize::MAX; self.num_edges].into_boxed_slice();
        self.start = vec![0; self.num_nodes + 1].into_boxed_slice();
        self.to = vec![usize::MAX; self.num_edges * 2].into_boxed_slice();
        self.flow = vec![Flow::zero(); self.num_edges * 2].into_boxed_slice();
        self.upper = vec![Flow::zero(); self.num_edges * 2].into_boxed_slice();
        self.rev = vec![usize::MAX; self.num_edges * 2].into_boxed_slice();
        self.distances = vec![self.num_nodes; self.num_nodes].into_boxed_slice();

        let mut degree = vec![0; self.num_nodes];
        for edge in graph.edges.iter() {
            degree[edge.to] += 1;
            degree[edge.from] += 1;
        }

        for i in 1..=self.num_nodes {
            self.start[i] += self.start[i - 1] + degree[i - 1];
        }

        let mut counter = vec![0; self.num_nodes];
        for (edge_index, e) in graph.edges.iter().enumerate() {
            let (u, v) = (e.from, e.to);
            let inside_edge_index_u = self.start[u] + counter[u];
            counter[u] += 1;
            let inside_edge_index_v = self.start[v] + counter[v];
            self.edge_index_to_inside_edge_index[edge_index] = inside_edge_index_u;
            counter[v] += 1;

            // u -> v
            self.to[inside_edge_index_u] = v;
            self.flow[inside_edge_index_u] = Flow::zero();
            self.upper[inside_edge_index_u] = e.upper;
            self.rev[inside_edge_index_u] = inside_edge_index_v;

            // v -> u
            self.to[inside_edge_index_v] = u;
            self.flow[inside_edge_index_v] = e.upper;
            self.upper[inside_edge_index_v] = e.upper;
            self.rev[inside_edge_index_v] = inside_edge_index_u;
        }
    }

    pub fn set_flow(&self, graph: &mut Graph<Flow>) {
        for edge_id in 0..graph.num_edges() {
            let i = self.edge_index_to_inside_edge_index[edge_id];
            graph.edges[edge_id].flow = self.flow[i];
        }
    }

    #[inline]
    pub fn neighbors(&self, u: usize) -> std::ops::Range<usize> {
        self.start[u]..self.start[u + 1]
    }

    #[inline]
    pub fn push_flow(&mut self, i: usize, flow: Flow) {
        let rev = self.rev[i];
        self.flow[i] += flow;
        self.flow[rev] -= flow;
    }

    // O(n + m)
    // calculate the distance from u to sink in the residual network
    // if such a path does not exist, distance[u] becomes self.num_nodes
    pub fn update_distances(&mut self, source: usize, sink: usize) {
        self.que.clear();
        self.que.push_back(sink);
        self.distances.fill(self.num_nodes);
        self.distances[sink] = 0;

        while let Some(v) = self.que.pop_front() {
            for i in self.neighbors(v) {
                // e.to -> v
                let to = self.to[i];
                if self.flow[i] > Flow::zero() && self.distances[to] == self.num_nodes {
                    self.distances[to] = self.distances[v] + 1;
                    if to != source {
                        self.que.push_back(to);
                    }
                }
            }
        }
    }

    #[inline]
    pub fn is_admissible_edge(&self, from: usize, i: usize) -> bool {
        self.residual_capacity(i) > Flow::zero() && self.distances[from] == self.distances[self.to[i]] + 1
    }

    pub fn residual_capacity(&self, i: usize) -> Flow {
        self.upper[i] - self.flow[i]
    }
}
