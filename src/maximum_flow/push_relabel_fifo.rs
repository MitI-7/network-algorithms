use crate::maximum_flow::csr::CSR;
use crate::maximum_flow::graph::Graph;
use crate::maximum_flow::status::Status;
use crate::maximum_flow::MaximumFlowSolver;
use num_traits::NumAssign;
use std::collections::VecDeque;

#[derive(Default)]
pub struct PushRelabelFIFO<Flow> {
    csr: CSR<Flow>,

    alpha: usize,
    relabel_count: usize,
    active_nodes: VecDeque<usize>,
    current_edge: Vec<usize>,
    distance_count: Vec<usize>,
}

impl<Flow> MaximumFlowSolver<Flow> for PushRelabelFIFO<Flow>
where
    Flow: NumAssign + Ord + Copy + Default,
{
    fn solve(&mut self, graph: &mut Graph<Flow>, source: usize, sink: usize, upper: Option<Flow>) -> Result<Flow, Status> {
        if source >= graph.num_nodes() || sink >= graph.num_nodes() || source == sink {
            return Err(Status::BadInput);
        }

        self.csr.build(graph);

        self.pre_process(source, sink);
        while let Some(u) = self.active_nodes.pop_front() {
            // no path to sink
            if u == source || u == sink || self.csr.distances_to_sink[u] >= self.csr.num_nodes {
                continue;
            }
            self.discharge(u);

            if self.alpha != 0 && self.relabel_count > self.alpha * self.csr.num_nodes {
                self.relabel_count = 0;
                self.csr.update_distances_to_sink(source, sink);
            }
        }

        self.push_flow_excess_back_to_source(source, sink);

        self.csr.set_flow(graph);

        Ok(self.csr.excesses[sink])
    }
}

impl<Flow> PushRelabelFIFO<Flow>
where
    Flow: NumAssign + Ord + Copy + Default,
{
    pub fn new(&mut self, alpha: usize) -> Self {
        Self { alpha, ..Default::default() }
    }

    pub fn solve(&mut self, graph: &mut Graph<Flow>, source: usize, sink: usize, upper: Option<Flow>) -> Result<Flow, Status> {
        <Self as MaximumFlowSolver<Flow>>::solve(self, graph, source, sink, upper)
    }

    fn pre_process(&mut self, source: usize, sink: usize) {
        self.csr.excesses.fill(Flow::zero());
        self.current_edge.resize(self.csr.num_nodes, 0);
        self.distance_count.resize(self.csr.num_nodes + 1, 0);

        self.csr.update_distances_to_sink(source, sink);
        self.csr.distances_to_sink[source] = self.csr.num_nodes;

        for u in 0..self.csr.num_nodes {
            self.distance_count[self.csr.distances_to_sink[u]] += 1;
            self.current_edge[u] = self.csr.start[u];
        }

        for i in self.csr.start[source]..self.csr.start[source + 1] {
            let delta = self.csr.residual_capacity(i);
            self.csr.push_flow(source, i, delta, true);
            self.csr.excesses[self.csr.to[i]] += delta;
        }

        for u in 0..self.csr.num_nodes {
            if u != source && u != sink && self.csr.excesses[u] > Flow::zero() {
                self.active_nodes.push_back(u);
            }
        }
    }

    fn discharge(&mut self, u: usize) {
        // push
        for i in self.csr.neighbors(u) {
            self.current_edge[u] = i;
            if self.csr.excesses[u] > Flow::zero() {
                self.push(u, i);
            }

            if self.csr.excesses[u] == Flow::zero() {
                return;
            }
        }
        self.current_edge[u] = self.csr.start[u];

        // relabel
        // if self.distance_count[self.csr.distances[u]] == 1 {
        //     self.gap_relabeling(self.csr.distances[u]);
        // } else {
        self.relabel(u);
        // }

        if self.csr.excesses[u] > Flow::zero() {
            self.active_nodes.push_back(u);
        }
    }

    // push from u
    fn push(&mut self, u: usize, i: usize) {
        let to = self.csr.to[i];
        let delta = self.csr.excesses[u].min(self.csr.residual_capacity(i));
        if self.csr.is_admissible_edge(u, i) && delta > Flow::zero() {
            self.csr.push_flow(u, i, delta, false);
            if self.csr.excesses[to] == delta {
                self.active_nodes.push_back(to);
            }
        }
    }

    fn relabel(&mut self, u: usize) {
        self.relabel_count += 1;
        // self.distance_count[self.csr.distances[u]] -= 1;

        let new_distance = self
            .csr
            .neighbors(u)
            .filter(|&i| self.csr.residual_capacity(i) > Flow::zero())
            .map(|i| self.csr.distances_to_sink[self.csr.to[i]] + 1)
            .min()
            .unwrap()
            .min(self.csr.num_nodes);

        self.csr.distances_to_sink[u] = new_distance;
        self.distance_count[self.csr.distances_to_sink[u]] += 1;
    }

    // gap relabeling heuristic
    // set distance[u] >= k to distance[u] = n
    // O(n)
    fn gap_relabeling(&mut self, k: usize) {
        for u in 0..self.csr.num_nodes {
            if self.csr.distances_to_sink[u] >= k {
                self.distance_count[self.csr.distances_to_sink[u]] -= 1;
                self.csr.distances_to_sink[u] = self.csr.distances_to_sink[u].max(self.csr.num_nodes);
                self.distance_count[self.csr.distances_to_sink[u]] += 1;
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
