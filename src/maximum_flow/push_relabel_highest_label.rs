use crate::maximum_flow::csr::CSR;
use crate::maximum_flow::graph::Graph;
use crate::maximum_flow::status::Status;
use crate::maximum_flow::MaximumFlowSolver;
use num_traits::NumAssign;

#[derive(Default)]
pub struct PushRelabelHighestLabel<Flow> {
    csr: CSR<Flow>,
    current_edge: Vec<usize>,
    alpha: usize,
    relabel_count: usize,

    buckets: Vec<Vec<usize>>, // buckets[i] = active nodes with distance i
    in_bucket: Vec<bool>,
    bucket_idx: usize,

    distance_count: Vec<usize>,
    excesses: Vec<Flow>,
}

impl<Flow> MaximumFlowSolver<Flow> for PushRelabelHighestLabel<Flow>
where
    Flow: NumAssign + Ord + Copy + Default,
{
    fn solve(&mut self, graph: &mut Graph<Flow>, source: usize, sink: usize, upper: Option<Flow>) -> Result<Flow, Status> {
        if source >= graph.num_nodes() || sink >= graph.num_nodes() || source == sink {
            return Err(Status::BadInput);
        }

        self.csr.build(graph);
        self.pre_process(source, sink);

        self.alpha = 1;

        loop {
            if self.buckets[self.bucket_idx].is_empty() {
                if self.bucket_idx == 0 {
                    break;
                }
                self.bucket_idx -= 1;
                continue;
            }

            let u = self.buckets[self.bucket_idx].pop().unwrap();
            self.in_bucket[u] = false;
            self.discharge(u);

            // if self.alpha != 0 && self.relabel_count > self.alpha * self.csr.num_nodes {
            //     self.relabel_count = 0;
            //     self.csr.update_distances(source, sink);
            // }
        }

        self.push_flow_excess_back_to_source(source, sink);

        self.csr.set_flow(graph);

        Ok(self.excesses[sink])
    }
}

impl<Flow> PushRelabelHighestLabel<Flow>
where
    Flow: NumAssign + Ord + Copy + Default,
{
    pub fn new(alpha: usize) -> Self {
        Self {
            csr: CSR::default(),
            excesses: Vec::new(),
            current_edge: Vec::new(),
            alpha,
            relabel_count: 0,
            buckets: Vec::new(),
            in_bucket: Vec::new(),
            bucket_idx: 0,
            distance_count: Vec::new(),
        }
    }

    pub fn solve(&mut self, graph: &mut Graph<Flow>, source: usize, sink: usize, upper: Option<Flow>) -> Result<Flow, Status> {
        <Self as MaximumFlowSolver<Flow>>::solve(self, graph, source, sink, upper)
    }

    fn pre_process(&mut self, source: usize, sink: usize) {
        self.excesses = vec![Flow::zero(); self.csr.num_nodes];
        self.current_edge = vec![0; self.csr.num_nodes];
        self.buckets = vec![Vec::new(); self.csr.num_nodes];
        self.in_bucket = vec![false; self.csr.num_nodes];
        self.distance_count = vec![0; self.csr.num_nodes + 1];

        self.csr.update_distances_to_sink(source, sink);
        self.csr.distances_to_sink[source] = self.csr.num_nodes;

        for u in 0..self.csr.num_nodes {
            self.distance_count[self.csr.distances_to_sink[u]] += 1;
            self.current_edge[u] = self.csr.start[u];
        }

        for i in self.csr.start[source]..self.csr.start[source + 1] {
            let delta = self.csr.residual_capacity(i);
            self.csr.push_flow(i, delta);
            self.excesses[self.csr.to[i]] += delta;
        }

        for u in 0..self.csr.num_nodes {
            if u != source && u != sink && self.excesses[u] > Flow::zero() {
                self.enqueue(u);
            }
        }

        self.in_bucket[sink] = true;
    }

    fn enqueue(&mut self, u: usize) {
        if self.in_bucket[u] || self.excesses[u] <= Flow::zero() || self.csr.distances_to_sink[u] >= self.csr.num_nodes {
            return;
        }

        self.in_bucket[u] = true;
        self.buckets[self.csr.distances_to_sink[u]].push(u);
        self.bucket_idx = self.bucket_idx.max(self.csr.distances_to_sink[u]);
        self.current_edge[u] = self.csr.start[u];
    }

    fn discharge(&mut self, u: usize) {
        // push
        for i in self.current_edge[u]..self.csr.start[u + 1] {
            self.current_edge[u] = i;
            if self.excesses[u] > Flow::zero() {
                self.push(u, i);
            }

            if self.excesses[u] == Flow::zero() {
                return;
            }
        }

        // relabel
        if self.distance_count[self.csr.distances_to_sink[u]] == 1 {
            self.gap_relabeling(self.csr.distances_to_sink[u]);
        } else {
            self.relabel(u);
        }
    }

    fn push(&mut self, u: usize, i: usize) {
        let to = self.csr.to[i];
        let delta = self.excesses[u].min(self.csr.residual_capacity(i));
        if self.csr.is_admissible_edge(u, i) && delta > Flow::zero() {
            self.csr.push_flow(i, delta);
            self.excesses[u] -= delta;
            self.excesses[to] += delta;
            self.enqueue(to);
        }
    }

    fn relabel(&mut self, u: usize) {
        self.relabel_count += 1;
        self.distance_count[self.csr.distances_to_sink[u]] -= 1;

        self.csr.distances_to_sink[u] = self
            .csr
            .neighbors(u)
            .filter(|&i| self.csr.residual_capacity(i) > Flow::zero())
            .map(|i| self.csr.distances_to_sink[self.csr.to[i]] + 1)
            .min()
            .unwrap_or(self.csr.num_nodes)
            .min(self.csr.num_nodes);

        self.distance_count[self.csr.distances_to_sink[u]] += 1;
        self.enqueue(u);
    }

    // gap relabeling heuristic
    fn gap_relabeling(&mut self, k: usize) {
        for u in 0..self.csr.num_nodes {
            if self.csr.distances_to_sink[u] >= k {
                self.distance_count[self.csr.distances_to_sink[u]] -= 1;
                self.csr.distances_to_sink[u] = self.csr.distances_to_sink[u].max(self.csr.num_nodes);
                self.distance_count[self.csr.distances_to_sink[u]] += 1;
                self.enqueue(u);
            }
        }
    }

    fn push_flow_excess_back_to_source(&mut self, source: usize, sink: usize) {
        for u in 0..self.csr.num_nodes {
            if u == source || u == sink {
                continue;
            }
            while self.excesses[u] > Flow::zero() {
                let mut visited = vec![false; self.csr.num_nodes];
                self.current_edge.iter_mut().enumerate().for_each(|(u, e)| *e = self.csr.start[u]);
                let d = self.dfs(u, source, self.excesses[u], &mut visited);
                self.excesses[u] -= d;
                self.excesses[source] += d;
            }
        }
    }

    fn dfs(&mut self, u: usize, source: usize, flow: Flow, visited: &mut Vec<bool>) -> Flow {
        if u == source {
            return flow;
        }
        visited[u] = true;

        for i in self.current_edge[u]..self.csr.start[u + 1] {
            self.current_edge[u] = i;
            let to = self.csr.to[i];
            let residual_capacity = self.csr.residual_capacity(i);
            if visited[to] || residual_capacity == Flow::zero() {
                continue;
            }

            let delta = self.dfs(to, source, flow.min(residual_capacity), visited);
            if delta > Flow::zero() {
                self.csr.push_flow(i, delta);
                return delta;
            }
        }
        Flow::zero()
    }
}
