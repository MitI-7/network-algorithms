use crate::{
    algorithms::maximum_flow::{
        edge::MaximumFlowEdge,
        error::MaximumFlowError,
        residual_network::ResidualNetwork,
        solvers::{macros::impl_maximum_flow_solver, solver::MaximumFlowSolver},
        status::Status,
        validate::validate_input,
    },
    core::numeric::FlowNum,
    graph::{
        direction::Directed,
        graph::Graph,
        ids::{ArcId, EdgeId, NodeId},
    },
};

pub struct PushRelabelHighestLabel<F> {
    status: Status,
    source: Option<NodeId>,

    rn: ResidualNetwork<F>,
    current_arc: Vec<usize>,

    global_relabel_freq: f64,
    value_only: bool,
    threshold: usize,
    work: usize,

    buckets: Box<[Vec<NodeId>]>, // buckets[i] = active nodes with distance i
    in_bucket: Box<[bool]>,
    bucket_idx: usize,

    distance_count: Vec<usize>,
}

impl<F> PushRelabelHighestLabel<F>
where
    F: FlowNum,
{
    fn new<N>(graph: &Graph<Directed, N, MaximumFlowEdge<F>>) -> Self {
        let rn = ResidualNetwork::new(graph);
        let num_nodes = rn.num_nodes;
        Self {
            status: Status::NotSolved,
            source: None,
            rn,
            current_arc: Vec::new(),

            global_relabel_freq: 1.0,
            value_only: false,
            threshold: 0,
            work: 0,

            buckets: vec![Vec::new(); num_nodes].into_boxed_slice(),
            in_bucket: vec![false; num_nodes].into_boxed_slice(),
            bucket_idx: 0,

            distance_count: Vec::new(),
        }
    }

    pub fn set_value_only(mut self, value_only: bool) -> Self {
        self.value_only = value_only;
        self
    }

    pub fn set_global_relabel_freq(mut self, global_relabel_freq: f64) -> Self {
        self.global_relabel_freq = global_relabel_freq;
        self
    }

    fn run(&mut self, source: NodeId, sink: NodeId) -> Result<F, MaximumFlowError> {
        validate_input(&self.rn, source, sink)?;

        self.source = Some(source);
        self.pre_process(source, sink);
        loop {
            if self.buckets[self.bucket_idx].is_empty() {
                if self.bucket_idx == 0 {
                    break;
                }
                self.bucket_idx -= 1;
                continue;
            }

            let u = self.buckets[self.bucket_idx].pop().unwrap();
            self.in_bucket[u.index()] = false;
            self.discharge(u);

            if self.work > self.threshold {
                self.work = 0;
                self.rn.update_distances_to_sink(source, sink);
                self.distance_count.fill(0);
                for u in 0..self.rn.num_nodes {
                    self.distance_count[self.rn.distances_to_sink[u]] += 1;
                }
            }
        }

        if !self.value_only {
            self.push_flow_excess_back_to_source(source, sink);
        }

        self.status = Status::Optimal;
        Ok(self.rn.excesses[sink.index()])
    }

    fn pre_process(&mut self, source: NodeId, sink: NodeId) {
        self.rn.excesses.fill(F::zero());
        self.current_arc = vec![0; self.rn.num_nodes];
        self.buckets.fill(Vec::new());
        self.in_bucket.fill(false);
        self.distance_count = vec![0; self.rn.num_nodes + 1];

        self.rn.update_distances_to_sink(source, sink);
        self.rn.distances_to_sink[source.index()] = self.rn.num_nodes;

        for u in 0..self.rn.num_nodes {
            self.distance_count[self.rn.distances_to_sink[u]] += 1;
            self.current_arc[u] = self.rn.start[u];
        }

        for arc_id in self.rn.neighbors(source) {
            let delta = self.rn.residual_capacity(arc_id);
            self.rn.push_flow_without_excess(source, arc_id, delta);
            self.rn.excesses[self.rn.to[arc_id.index()].index()] += delta;
        }

        for u in (0..self.rn.num_nodes).map(NodeId) {
            if u != source && u != sink && self.rn.excesses[u.index()] > F::zero() {
                self.enqueue(u);
            }
        }

        self.in_bucket[sink.index()] = true;

        self.threshold = if self.global_relabel_freq <= 0.0 {
            usize::MAX
        } else {
            (((self.rn.num_nodes + self.rn.num_edges) as f64) / self.global_relabel_freq).ceil() as usize
        };
    }

    fn enqueue(&mut self, u: NodeId) {
        if self.in_bucket[u.index()]
            || self.rn.excesses[u.index()] <= F::zero()
            || self.rn.distances_to_sink[u.index()] >= self.rn.num_nodes
        {
            return;
        }

        self.in_bucket[u.index()] = true;
        self.buckets[self.rn.distances_to_sink[u.index()]].push(u);
        self.bucket_idx = self.bucket_idx.max(self.rn.distances_to_sink[u.index()]);
        self.current_arc[u.index()] = self.rn.start[u.index()];
    }

    fn discharge(&mut self, u: NodeId) {
        // push
        for arc_id in (self.current_arc[u.index()]..self.rn.start[u.index() + 1]).map(ArcId) {
            self.current_arc[u.index()] = arc_id.index();
            if self.rn.excesses[u.index()] > F::zero() {
                self.push(u, arc_id);
            }

            if self.rn.excesses[u.index()] == F::zero() {
                return;
            }
        }

        // relabel
        if self.distance_count[self.rn.distances_to_sink[u.index()]] == 1 {
            self.gap_relabeling(self.rn.distances_to_sink[u.index()]);
        } else {
            self.relabel(u);
        }
    }

    fn push(&mut self, u: NodeId, arc_id: ArcId) {
        let to = self.rn.to[arc_id.index()];
        let delta = self.rn.excesses[u.index()].min(self.rn.residual_capacity(arc_id));
        if self.rn.is_admissible_arc(u, arc_id) && delta > F::zero() {
            self.rn.push_flow(u, arc_id, delta);
            self.enqueue(to);
        }
    }

    fn relabel(&mut self, u: NodeId) {
        self.work += self.rn.start[u.index() + 1] - self.rn.start[u.index()]; // add outdegree of u
        self.distance_count[self.rn.distances_to_sink[u.index()]] -= 1;

        self.rn.distances_to_sink[u.index()] = self
            .rn
            .neighbors(u)
            .filter(|&i| self.rn.residual_capacity(i) > F::zero())
            .map(|i| self.rn.distances_to_sink[self.rn.to[i.index()].index()] + 1)
            .min()
            .unwrap_or(self.rn.num_nodes)
            .min(self.rn.num_nodes);

        self.distance_count[self.rn.distances_to_sink[u.index()]] += 1;
        self.enqueue(u);
    }

    // gap relabeling heuristic
    fn gap_relabeling(&mut self, k: usize) {
        for u in 0..self.rn.num_nodes {
            if self.rn.distances_to_sink[u] >= k {
                self.distance_count[self.rn.distances_to_sink[u]] -= 1;
                self.rn.distances_to_sink[u] = self.rn.distances_to_sink[u].max(self.rn.num_nodes);
                self.distance_count[self.rn.distances_to_sink[u]] += 1;
                self.enqueue(NodeId(u));
            }
        }
    }

    fn push_flow_excess_back_to_source(&mut self, source: NodeId, sink: NodeId) {
        for u in (0..self.rn.num_nodes).map(NodeId) {
            if u == source || u == sink {
                continue;
            }
            while self.rn.excesses[u.index()] > F::zero() {
                let mut visited = vec![false; self.rn.num_nodes];
                self.current_arc
                    .iter_mut()
                    .enumerate()
                    .for_each(|(u, e)| *e = self.rn.start[u]);
                let d = self.dfs(u, source, self.rn.excesses[u.index()], &mut visited);
                self.rn.excesses[u.index()] -= d;
                self.rn.excesses[source.index()] += d;
            }
        }
    }

    fn dfs(&mut self, u: NodeId, source: NodeId, flow: F, visited: &mut Vec<bool>) -> F {
        if u == source {
            return flow;
        }
        visited[u.index()] = true;

        for arc_id in (self.current_arc[u.index()]..self.rn.start[u.index() + 1]).map(ArcId) {
            self.current_arc[u.index()] = arc_id.index();
            let to = self.rn.to[arc_id.index()];
            let residual_capacity = self.rn.residual_capacity(arc_id);
            if visited[to.index()] || residual_capacity == F::zero() {
                continue;
            }

            let delta = self.dfs(to, source, flow.min(residual_capacity), visited);
            if delta > F::zero() {
                self.rn.push_flow_without_excess(u, arc_id, delta);
                return delta;
            }
        }
        F::zero()
    }
}

impl_maximum_flow_solver!(PushRelabelHighestLabel, run);
