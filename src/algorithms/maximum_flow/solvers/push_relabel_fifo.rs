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
use std::collections::VecDeque;

pub struct PushRelabelFifo<F> {
    status: Status,
    source: Option<NodeId>,

    rn: ResidualNetwork<F>,
    global_relabel_freq: f64,
    value_only: bool,
    threshold: usize,
    work: usize,
    active_nodes: VecDeque<NodeId>,
    current_edge: Box<[usize]>,
    distance_count: Box<[usize]>,
}

impl<F> PushRelabelFifo<F>
where
    F: FlowNum,
{
    fn new<N>(graph: &Graph<Directed, N, MaximumFlowEdge<F>>) -> Self {
        let rn = ResidualNetwork::from(graph);
        let num_nodes = rn.num_nodes;

        Self {
            status: Status::NotSolved,
            source: None,
            rn,
            global_relabel_freq: 1.0,
            value_only: false,
            threshold: 0,
            work: 0,
            active_nodes: VecDeque::new(),
            current_edge: vec![0_usize; num_nodes].into_boxed_slice(),
            distance_count: vec![0_usize; num_nodes + 1].into_boxed_slice(),
        }
    }

    fn run(&mut self, source: NodeId, sink: NodeId) -> Result<F, MaximumFlowError> {
        validate_input(&self.rn, source, sink)?;

        // initialize
        self.source = Some(source);
        self.rn.residual_capacities.copy_from_slice(&self.rn.upper);
        self.rn.excesses.fill(F::zero());

        let residual = self
            .rn
            .neighbors(source)
            .fold(F::zero(), |sum, arc_id| sum + self.rn.upper[arc_id.index()]);

        self.rn.excesses[source.index()] = residual;

        self.pre_process(source, sink);
        while let Some(u) = self.active_nodes.pop_front() {
            // no path to sink
            if u == source || u == sink || self.rn.distances_to_sink[u.index()] >= self.rn.num_nodes {
                continue;
            }
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

    pub fn set_value_only(mut self, value_only: bool) -> Self {
        self.value_only = value_only;
        self
    }

    pub fn set_global_relabel_freq(mut self, global_relabel_freq: f64) -> Self {
        self.global_relabel_freq = global_relabel_freq;
        self
    }

    fn pre_process(&mut self, source: NodeId, sink: NodeId) {
        self.current_edge.fill(0);
        self.distance_count.fill(0);

        self.rn.update_distances_to_sink(source, sink);
        self.rn.distances_to_sink[source.index()] = self.rn.num_nodes;

        for u in 0..self.rn.num_nodes {
            self.distance_count[self.rn.distances_to_sink[u]] += 1;
            self.current_edge[u] = self.rn.start[u];
        }

        for arc_id in self.rn.neighbors(source) {
            let delta = self.rn.residual_capacities[arc_id.index()];
            self.rn.push_flow_without_excess(source, arc_id, delta);
            self.rn.excesses[self.rn.to[arc_id.index()].index()] += delta;
        }

        for u in 0..self.rn.num_nodes {
            let u = NodeId(u);
            if u != source && u != sink && self.rn.excesses[u.index()] > F::zero() {
                self.active_nodes.push_back(u);
            }
        }

        self.threshold = if self.global_relabel_freq <= 0.0 {
            usize::MAX
        } else {
            (((self.rn.num_nodes + self.rn.num_edges) as f64) / self.global_relabel_freq).ceil() as usize
        };
    }

    fn discharge(&mut self, u: NodeId) {
        // push
        // for arc_id in self.rn.neighbors(u) {
        for arc_id in self.current_edge[u.index()]..self.rn.start[u.index() + 1] {
            let arc_id = ArcId(arc_id);
            self.current_edge[u.index()] = arc_id.index();
            if self.rn.excesses[u.index()] > F::zero() {
                self.push(u, arc_id);
            }

            if self.rn.excesses[u.index()] == F::zero() {
                return;
            }
        }
        self.current_edge[u.index()] = self.rn.start[u.index()];

        // relabel
        if self.distance_count[self.rn.distances_to_sink[u.index()]] == 1 {
            self.gap_relabeling(self.rn.distances_to_sink[u.index()]);
        } else {
            self.relabel(u);
        }

        if self.rn.excesses[u.index()] > F::zero() {
            self.active_nodes.push_back(u);
        }
    }

    // push from u
    fn push(&mut self, u: NodeId, arc_id: ArcId) {
        let to = self.rn.to[arc_id.index()];
        let delta = self.rn.excesses[u.index()].min(self.rn.residual_capacities[arc_id.index()]);
        if self.rn.is_admissible_arc(u, arc_id) && delta > F::zero() {
            self.rn.push_flow(u, arc_id, delta);
            if self.rn.excesses[to.index()] == delta {
                self.active_nodes.push_back(to);
            }
        }
    }

    fn relabel(&mut self, u: NodeId) {
        self.work += self.rn.start[u.index() + 1] - self.rn.start[u.index()]; // add outdegree of u
        self.distance_count[self.rn.distances_to_sink[u.index()]] -= 1;

        let new_distance = self
            .rn
            .neighbors(u)
            .filter(|&arc_id| self.rn.residual_capacities[arc_id.index()] > F::zero())
            .map(|arc_id| self.rn.distances_to_sink[self.rn.to[arc_id.index()].index()] + 1)
            .min()
            .expect("relabel: no outgoing residual arc found")
            .min(self.rn.num_nodes);

        self.rn.distances_to_sink[u.index()] = new_distance;
        self.distance_count[self.rn.distances_to_sink[u.index()]] += 1;
    }

    // gap relabeling heuristic
    // set distance[u] >= k to distance[u] = n
    // O(n)
    fn gap_relabeling(&mut self, k: usize) {
        for u in 0..self.rn.num_nodes {
            if self.rn.distances_to_sink[u] >= k {
                self.distance_count[self.rn.distances_to_sink[u]] -= 1;
                self.rn.distances_to_sink[u] = self.rn.distances_to_sink[u].max(self.rn.num_nodes);
                self.distance_count[self.rn.distances_to_sink[u]] += 1;
            }
        }
    }

    fn push_flow_excess_back_to_source(&mut self, source: NodeId, sink: NodeId) {
        let mut visited = vec![false; self.rn.num_nodes].into_boxed_slice();
        for u in 0..self.rn.num_nodes {
            let u = NodeId(u);
            if u == source || u == sink {
                continue;
            }
            while self.rn.excesses[u.index()] > F::zero() {
                visited.fill(false);
                self.current_edge
                    .iter_mut()
                    .enumerate()
                    .for_each(|(u, e)| *e = self.rn.start[u]);
                let d = self.dfs(u, source, self.rn.excesses[u.index()], &mut visited);
                self.rn.excesses[u.index()] -= d;
                self.rn.excesses[source.index()] += d;
            }
        }
    }

    fn dfs(&mut self, u: NodeId, source: NodeId, flow: F, visited: &mut [bool]) -> F {
        if u == source {
            return flow;
        }
        visited[u.index()] = true;

        for i in self.current_edge[u.index()]..self.rn.start[u.index() + 1] {
            let i = ArcId(i);
            self.current_edge[u.index()] = i.index();
            let to = self.rn.to[i.index()];
            let residual_capacity = self.rn.residual_capacities[i.index()];
            if visited[to.index()] || residual_capacity == F::zero() {
                continue;
            }

            let delta = self.dfs(to, source, flow.min(residual_capacity), visited);
            if delta > F::zero() {
                self.rn.push_flow(u, i, delta);
                return delta;
            }
        }
        F::zero()
    }
}

impl_maximum_flow_solver!(PushRelabelFifo, run);
