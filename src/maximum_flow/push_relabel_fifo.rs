use crate::maximum_flow::csr::CSR;
use crate::maximum_flow::graph::Graph;
use crate::maximum_flow::status::Status;
use num_traits::NumAssign;
use std::collections::VecDeque;

#[derive(Default)]
pub struct PushRelabelFIFO<Flow> {
    csr: CSR<Flow>,
    excesses: Vec<Flow>,

    alpha: usize,
    relabel_count: usize,
    active_nodes: VecDeque<usize>,
    current_edge: Vec<usize>,
    distance_count: Vec<usize>,
}

impl<Flow> PushRelabelFIFO<Flow>
where
    Flow: NumAssign + Ord + Copy + Default,
{
    pub fn new(&mut self, alpha: usize) -> Self {
        Self { alpha, ..Default::default() }
    }

    pub fn solve(&mut self, graph: &mut Graph<Flow>, source: usize, sink: usize, upper: Option<Flow>) -> Result<Flow, Status> {
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

        Ok(self.excesses[sink])
    }

    fn pre_process(&mut self, source: usize, sink: usize) {
        self.excesses.resize(self.csr.num_nodes, Flow::zero());
        self.current_edge.resize(self.csr.num_nodes, 0);
        self.distance_count.resize(self.csr.num_nodes + 1, 0);

        self.csr.update_distances_to_sink(source, sink);
        self.csr.distances_to_sink[source] = self.csr.num_nodes;

        for u in 0..self.csr.num_nodes {
            self.distance_count[self.csr.distances_to_sink[u]] += 1;
            self.current_edge[u] = self.csr.start[u];
        }

        for i in self.csr.start[source]..self.csr.start[source + 1] {
            let to = self.csr.to[i];
            let delta = self.csr.residual_capacity(i);
            self.excesses[to] += delta;
            self.csr.push_flow(i, delta);
        }

        for u in 0..self.csr.num_nodes {
            if u != source && u != sink && self.excesses[u] > Flow::zero() {
                self.active_nodes.push_back(u);
            }
        }
    }

    fn discharge(&mut self, u: usize) {
        // push
        for i in self.csr.neighbors(u) {
            self.current_edge[u] = i;
            if self.excesses[u] > Flow::zero() {
                self.push(u, i);
            }

            if self.excesses[u] == Flow::zero() {
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

        if self.excesses[u] > Flow::zero() {
            self.active_nodes.push_back(u);
        }
    }

    // push from u
    fn push(&mut self, u: usize, i: usize) {
        let to = self.csr.to[i];
        let delta = self.excesses[u].min(self.csr.residual_capacity(i));
        if self.csr.is_admissible_edge(u, i) && delta > Flow::zero() {
            self.csr.push_flow(i, delta);
            self.excesses[u] -= delta;
            self.excesses[to] += delta;
            if self.excesses[to] == delta {
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
