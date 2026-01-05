use std::collections::VecDeque;
use crate::{
    algorithms::maximum_flow::{
        algorithms::{macros::impl_maximum_flow_solver, solver::MaximumFlowSolver},
        edge::MaximumFlowEdge,
        residual_network::ResidualNetwork,
        result::{MaximumFlowResult, MinimumCutResult},
        status::Status,
        validate::validate_input,
    },
    core::numeric::FlowNum,
    graph::{direction::Directed, graph::Graph, ids::NodeId},
};

pub struct PushRelabelHighestLabel<Flow> {
    csr: CSR<Flow>,
    current_edge: Vec<usize>,

    global_relabel_freq: f64,
    value_only: bool,
    threshold: usize,
    work: usize,

    buckets: Vec<Vec<usize>>, // buckets[i] = active nodes with distance i
    in_bucket: Vec<bool>,
    bucket_idx: usize,

    distance_count: Vec<usize>,
}

impl<Flow> Default for PushRelabelHighestLabel<Flow>
where
    Flow: Default,
{
    fn default() -> Self {
        Self {
            csr: CSR::default(),
            current_edge: Vec::new(),

            global_relabel_freq: 1.0,
            value_only: false,
            threshold: 0,
            work: 0,

            buckets: Vec::new(),
            in_bucket: Vec::new(),
            bucket_idx: 0,

            distance_count: Vec::new(),
        }
    }
}

impl<Flow> MaximumFlowSolver<Flow> for PushRelabelHighestLabel<Flow>
where
    Flow: FlowNum,
{
    fn solve(&mut self, graph: &mut Graph<Directed, (), CapEdge<Flow>>, source: NodeId, sink: NodeId, upper: Option<Flow>) -> Result<Flow, Status> {
        if source.index() >= graph.num_nodes() || sink.index() >= graph.num_nodes() || source == sink {
            return Err(Status::BadInput);
        }

        let delta: Flow = graph.edges.iter().filter(|e| e.u == source).fold(Flow::zero(), |acc, e| acc + e.data.upper);
        let dummy_source = graph.add_node(); // dummy source
        graph.add_edge(dummy_source, source, CapEdge{flow: Flow::zero(), upper: upper.unwrap_or(delta)}); // dummy edge
        self.csr.build(graph);
        let source = dummy_source;

        self.pre_process(source.index(), sink.index());
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

            if self.work > self.threshold {
                self.work = 0;
                self.csr.update_distances_to_sink(source.index(), sink.index());
                self.distance_count.fill(0);
                for u in 0..self.csr.num_nodes {
                    self.distance_count[self.csr.distances_to_sink[u]] += 1;
                }
            }
        }

        if !self.value_only {
            self.push_flow_excess_back_to_source(source.index(), sink.index());
            self.csr.set_flow(graph);
        }

        // remove dummy source & dummy edge
        graph.pop_node();
        graph.pop_edge();

        Ok(self.csr.excesses[sink.index()])
    }
}

impl<Flow> PushRelabelHighestLabel<Flow>
where
    Flow: FlowNum,
{
    pub fn set_value_only(mut self, value_only: bool) -> Self {
        self.value_only = value_only;
        self
    }

    pub fn set_global_relabel_freq(mut self, global_relabel_freq: f64) -> Self {
        self.global_relabel_freq = global_relabel_freq;
        self
    }

    fn new<N>(graph: &Graph<Directed, N, MaximumFlowEdge<F>>) -> Self {
        let rn = ResidualNetwork::new(graph);
        let num_nodes = rn.num_nodes;

        Self {
            rn,
            current_edge: vec![0_usize; num_nodes].into_boxed_slice(),
            distances_to_sink: vec![0; num_nodes].into_boxed_slice(),
            que: VecDeque::new(),
            cutoff: None,
        }
    }

    pub fn solve(&mut self, graph: &mut Graph<Directed, (), CapEdge<Flow>>, source: NodeId, sink: NodeId, upper: Option<Flow>) -> Result<Flow, Status> {
        <Self as MaximumFlowSolver<Flow>>::solve(self, graph, source, sink, upper)
    }

    fn pre_process(&mut self, source: usize, sink: usize) {
        self.csr.excesses.fill(Flow::zero());
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

        for edge_id in self.csr.neighbors(source) {
            let delta = self.csr.residual_capacity(edge_id);
            self.csr.push_flow(source, edge_id, delta, true);
            self.csr.excesses[self.csr.to[edge_id]] += delta;
        }

        for u in 0..self.csr.num_nodes {
            if u != source && u != sink && self.csr.excesses[u] > Flow::zero() {
                self.enqueue(u);
            }
        }

        self.in_bucket[sink] = true;

        self.threshold = if self.global_relabel_freq <= 0.0 {
            usize::MAX
        } else {
            (((self.csr.num_nodes + self.csr.num_edges) as f64) / self.global_relabel_freq).ceil() as usize
        };
    }

    fn enqueue(&mut self, u: usize) {
        if self.in_bucket[u] || self.csr.excesses[u] <= Flow::zero() || self.csr.distances_to_sink[u] >= self.csr.num_nodes {
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
            if self.csr.excesses[u] > Flow::zero() {
                self.push(u, i);
            }

            if self.csr.excesses[u] == Flow::zero() {
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
        let delta = self.csr.excesses[u].min(self.csr.residual_capacity(i));
        if self.csr.is_admissible_edge(u, i) && delta > Flow::zero() {
            self.csr.push_flow(u, i, delta, false);
            self.enqueue(to);
        }
    }

    fn relabel(&mut self, u: usize) {
        self.work += self.csr.start[u + 1] - self.csr.start[u]; // add outdegree of u
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
            while self.csr.excesses[u] > Flow::zero() {
                let mut visited = vec![false; self.csr.num_nodes];
                self.current_edge.iter_mut().enumerate().for_each(|(u, e)| *e = self.csr.start[u]);
                let d = self.dfs(u, source, self.csr.excesses[u], &mut visited);
                self.csr.excesses[u] -= d;
                self.csr.excesses[source] += d;
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
                self.csr.push_flow(u, i, delta, true);
                return delta;
            }
        }
        Flow::zero()
    }
}

impl_maximum_flow_solver!(PushRelabelHighestLabel, run);