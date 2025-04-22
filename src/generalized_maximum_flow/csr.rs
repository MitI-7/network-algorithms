use crate::generalized_maximum_flow::graph::Graph;
use num_traits::{Float, ToPrimitive};
use std::cmp::Reverse;
use std::collections::{BinaryHeap, VecDeque};

pub type Dist = i32;

#[derive(Default)]
pub struct CSR<Flow> {
    pub num_nodes: usize,
    pub num_edges: usize,
    pub base: Flow,
    pub epsilon: Flow,
    pub edge_index_to_inside_edge_index: Box<[usize]>,
    pub is_lossy: bool,

    pub start: Box<[usize]>,
    pub from: Box<[usize]>,
    pub to: Box<[usize]>,
    pub flow: Box<[Flow]>,
    pub capacity: Box<[Flow]>,
    pub dist: Box<[Dist]>,
    pub rev_edge_id: Box<[usize]>,

    pub gains: Box<[Flow]>,

    pub excesses: Box<[Flow]>,
    pub potentials: Box<[Dist]>,
}

#[allow(dead_code)]
impl<Flow> CSR<Flow>
where
    Flow: Float + Copy + Clone + ToPrimitive + Default,
{
    pub fn new(epsilon: Flow) -> Self {
        Self { epsilon, ..Default::default() }
    }

    pub fn new_with_base(base: Flow) -> Self {
        Self { base, ..Default::default() }
    }

    pub fn build(&mut self, graph: &mut Graph<Flow>) {
        self.num_nodes = graph.num_nodes();
        self.num_edges = graph.num_edges();

        if self.base == Flow::zero() {
            self.base = (Flow::one() + self.epsilon).powf(Flow::one() / Flow::from(self.num_nodes).unwrap());
        }

        // initialize
        self.edge_index_to_inside_edge_index = vec![usize::MAX; self.num_edges].into_boxed_slice();
        self.gains = vec![Flow::zero(); self.num_edges].into_boxed_slice();
        self.start = vec![0; self.num_nodes + 1].into_boxed_slice();
        self.from = vec![0; self.num_edges * 2].into_boxed_slice();
        self.to = vec![0; self.num_edges * 2].into_boxed_slice();
        self.flow = vec![Flow::zero(); self.num_edges * 2].into_boxed_slice();
        self.capacity = vec![Flow::zero(); self.num_edges * 2].into_boxed_slice();
        self.dist = vec![0 as Dist; self.num_edges * 2].into_boxed_slice();
        self.rev_edge_id = vec![0; self.num_edges * 2].into_boxed_slice();
        self.excesses = vec![Flow::zero(); self.num_nodes].into_boxed_slice();
        self.potentials = vec![0 as Dist; self.num_nodes].into_boxed_slice();

        let mut degree = vec![0; self.num_nodes];
        for edge in graph.edges.iter() {
            degree[edge.to] += 1;
            degree[edge.from] += 1;
        }

        for i in 1..=self.num_nodes {
            self.start[i] += self.start[i - 1] + degree[i - 1];
        }

        let mut counter = vec![0; self.num_nodes];
        for (edge_index, edge) in graph.edges.iter().enumerate() {
            let (u, v) = (edge.from, edge.to);
            let inside_edge_index_u = self.start[u] + counter[u];
            counter[u] += 1;
            let inside_edge_index_v = self.start[v] + counter[v];
            counter[v] += 1;

            // gain scaling
            let c = edge.gain.log(self.base).floor();
            let scaled_gain = self.base.powf(c);
            // let dist = -1 * c as Dist; // TODO: check over flow
            let dist: Dist = -c.to_i32().unwrap();

            self.edge_index_to_inside_edge_index[edge_index] = inside_edge_index_u;
            self.gains[edge_index] = edge.gain;

            // u -> v
            self.from[inside_edge_index_u] = u;
            self.to[inside_edge_index_u] = v;
            self.flow[inside_edge_index_u] = Flow::zero();
            self.capacity[inside_edge_index_u] = edge.upper;
            self.dist[inside_edge_index_u] = dist;
            self.rev_edge_id[inside_edge_index_u] = inside_edge_index_v;

            // v -> u
            self.from[inside_edge_index_v] = v;
            self.to[inside_edge_index_v] = u;
            self.flow[inside_edge_index_v] = edge.upper * scaled_gain;
            self.capacity[inside_edge_index_v] = edge.upper * scaled_gain;
            self.dist[inside_edge_index_v] = -dist;
            self.rev_edge_id[inside_edge_index_v] = inside_edge_index_u;
        }
    }

    #[inline]
    pub fn neighbors(&self, u: usize) -> std::ops::Range<usize> {
        self.start[u]..self.start[u + 1]
    }

    #[inline]
    pub fn push_labeled_flow(&mut self, u: usize, i: usize, labeled_flow: Flow, labels: &[Flow]) {
        let to = self.to[i];
        let rev = self.rev_edge_id[i];

        self.flow[i] = self.flow[i] + labeled_flow * labels[u];
        self.flow[rev] = self.flow[rev] - labeled_flow * labels[to];

        self.excesses[u] = self.excesses[u] - labeled_flow * labels[u];
        self.excesses[to] = self.excesses[to] + labeled_flow * labels[to];

        if self.flow[i] > self.capacity[i] {
            self.flow[i] = self.capacity[i];
            self.flow[rev] = Flow::zero();
        }

        if self.flow[rev] < Flow::zero() {
            self.flow[rev] = Flow::zero();
            self.flow[i] = self.capacity[i];
        }

        let eps = Flow::from(1e-10).unwrap();

        if self.residual_capacity(i) <= eps || self.flow[rev] <= eps {
            self.flow[i] = self.capacity[i];
            self.flow[rev] = Flow::zero();
        }

        if self.excesses[u] <= eps {
            self.excesses[u] = Flow::zero();
        }
    }

    #[inline]
    pub fn push_flow(&mut self, u: usize, i: usize, flow: Flow, labels: &[Flow]) {
        let to = self.to[i];
        let rev = self.rev_edge_id[i];

        self.flow[i] = self.flow[i] + flow;
        self.flow[rev] = self.flow[rev] - flow * labels[to] / labels[u];

        self.excesses[u] = self.excesses[u] - flow;
        self.excesses[to] = self.excesses[to] + flow * labels[to] / labels[u];

        if self.flow[i] > self.capacity[i] {
            self.flow[i] = self.capacity[i];
            self.flow[rev] = Flow::zero();
        }

        if self.flow[rev] < Flow::zero() {
            self.flow[rev] = Flow::zero();
            self.flow[i] = self.capacity[i];
        }

        let eps = Flow::from(1e-10).unwrap();

        if self.residual_capacity(i) <= eps || self.flow[rev] <= eps {
            self.flow[i] = self.capacity[i];
            self.flow[rev] = Flow::zero();
        }

        if self.excesses[u] <= eps {
            self.excesses[u] = Flow::zero();
        }
    }

    pub fn calculate_distance_to_sink(&mut self, sink: usize) -> Vec<Dist> {
        let mut distance = vec![Dist::MAX; self.num_nodes];
        let mut distance_to_sink = vec![Dist::MAX; self.num_nodes];
        let mut visited = vec![false; self.num_nodes];
        distance[sink] = 0;
        distance_to_sink[sink] = 0;

        let mut heap = BinaryHeap::new();
        heap.push((Reverse(0), sink));

        let mut farthest = 0;
        while let Some((d, u)) = heap.pop() {
            if visited[u] {
                continue;
            }
            visited[u] = true;
            farthest = d.0;

            for i in self.neighbors(u) {
                let to = self.to[i];
                let flow = self.flow[i];

                if flow > Flow::zero() && !visited[to] {
                    let di = self.dist[i];

                    // using dist of edge(e.to -> u)
                    let dist = -di - self.potentials[to] + self.potentials[u];
                    assert!(dist >= 0);

                    let new_dist = d.0 + dist;
                    if new_dist < distance[to] {
                        distance[to] = new_dist;
                        distance_to_sink[to] = distance_to_sink[u] - di;
                        heap.push((Reverse(new_dist), to));
                    }
                }
            }
        }

        // update potentials
        self.potentials.iter_mut().enumerate().for_each(|(u, p)| *p += distance[u].min(farthest));
        distance_to_sink
    }

    pub fn calculate_distance_to_sink_with_negative_edge(&mut self, sink: usize) -> Option<Vec<Dist>> {
        let mut distance = vec![Dist::MAX; self.num_nodes];
        let mut distance_to_sink = vec![Dist::MAX; self.num_nodes];
        let mut in_queue = vec![false; self.num_nodes];
        let mut visit_count = vec![0; self.num_nodes];

        distance[sink] = 0;
        distance_to_sink[sink] = 0;

        let mut que = VecDeque::new();
        que.push_back(sink);
        in_queue[sink] = true;

        let mut farthest = 0;
        while let Some(u) = que.pop_front() {
            in_queue[u] = false;
            farthest = farthest.max(distance[u]);

            for i in self.neighbors(u) {
                let flow = self.flow[i];

                if flow > Flow::zero() {
                    let to = self.to[i];
                    let ed = self.dist[i];

                    let dist = -ed - self.potentials[to] + self.potentials[u];
                    let new_dist = distance[u] + dist;

                    if new_dist < distance[to] {
                        distance[to] = new_dist;
                        distance_to_sink[to] = distance_to_sink[u] - ed;

                        visit_count[to] += 1;
                        if visit_count[to] >= self.num_nodes {
                            // negative cycle detected
                            return None;
                        }

                        if !in_queue[to] {
                            in_queue[to] = true;
                            que.push_back(to);
                        }
                    }
                }
            }
        }

        self.potentials = self.potentials.iter().enumerate().map(|(u, p)| p + distance[u].min(farthest)).collect();
        Some(distance_to_sink)
    }

    #[inline]
    pub fn residual_capacity(&self, i: usize) -> Flow {
        self.capacity[i] - self.flow[i]
    }
}
