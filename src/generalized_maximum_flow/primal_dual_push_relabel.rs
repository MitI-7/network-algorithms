use crate::data_structures::bit_vector::BitVector;
use crate::generalized_maximum_flow::csr::{Dist, CSR};
use crate::generalized_maximum_flow::generalized_maximum_flow_graph::Graph;
use crate::generalized_maximum_flow::status::Status;
use num_traits::{Float, FromPrimitive, ToPrimitive};
use std::collections::VecDeque;

#[derive(Default)]
pub struct PrimalDualPushRelabel<Flow> {
    csr: CSR<Flow>,
    canonical_labels: Vec<Flow>,

    // maximum flow
    current_edge: Box<[usize]>,
    distances: Box<[usize]>,
    active_nodes: VecDeque<usize>,
    que: VecDeque<usize>,
    distance_count: Box<[usize]>,
    relabel_count: usize,
}

#[allow(dead_code)]
impl<Flow> PrimalDualPushRelabel<Flow>
where
    Flow: Float + PartialOrd + Copy + Clone + ToPrimitive + FromPrimitive + Default + std::fmt::Debug,
{
    pub fn new(epsilon: Flow) -> Self {
        Self { csr: CSR::new(epsilon), ..Default::default() }
    }

    pub fn solve(&mut self, source: usize, sink: usize, graph: &mut Graph<Flow>) -> Status {
        self.csr.build(graph);
        self.canonical_labels = vec![Flow::zero(); self.csr.num_nodes];
        self.current_edge = vec![0; self.csr.num_nodes].into_boxed_slice();
        self.distances = vec![0; self.csr.num_nodes].into_boxed_slice();
        self.distance_count = vec![0; self.csr.num_nodes + 1].into_boxed_slice();

        self.csr.excesses[source] = Flow::max_value();
        loop {
            let distance_to_sink = self.csr.calculate_distance_to_sink(sink);
            self.update_canonical_labels(&distance_to_sink, sink);

            // no augmenting path from source
            if self.canonical_labels[source] == Flow::max_value() {
                break;
            }

            // maximum flow
            self.pre_process(source, sink);
            while let Some(u) = self.active_nodes.pop_front() {
                if u == source || u == sink || self.distances[u] == self.csr.num_nodes {
                    continue;
                }
                assert!(self.csr.excesses[u] > Flow::zero());
                self.discharge(u);

                if self.relabel_count % 50000 == 0 {
                    self.relabel_count = 0;
                    self.update_distances(source, sink);
                }
            }

            self.push_flow_excess_back_to_source(source, sink);
            for u in 0..self.csr.num_nodes {
                if u != source && u != sink {
                    assert_eq!(self.csr.excesses[u], Flow::zero());
                }
            }
        }

        for u in 0..graph.num_nodes() {
            graph.excesses[u] = self.csr.excesses[u];
        }
        for edge_id in 0..graph.num_edges() {
            let i = self.csr.edge_index_to_inside_edge_index[edge_id];
            graph.edges[edge_id].flow = self.csr.flow[i];
        }

        Status::Optimal
    }

    fn update_canonical_labels(&mut self, distance_to_sink: &[Dist], sink: usize) {
        self.canonical_labels.iter_mut().enumerate().for_each(|(u, label)| {
            *label = if distance_to_sink[u] != Dist::MAX {
                self.csr.base.powf(Flow::from_i32(distance_to_sink[u]).unwrap())
            } else {
                Flow::max_value()
            }
        });
        self.canonical_labels[sink] = Flow::one();
    }

    fn pre_process(&mut self, source: usize, sink: usize) {
        self.update_distances(source, sink);
        self.distances[source] = self.csr.num_nodes;
        self.distance_count.fill(0);

        for u in 0..self.csr.num_nodes {
            self.distance_count[self.distances[u]] += 1;
            self.current_edge[u] = self.csr.start[u];
        }

        for i in self.csr.start[source]..self.csr.start[source + 1] {
            let delta = self.csr.residual_capacity(i);
            if self.csr.residual_capacity(i) > Flow::zero() && self.reduced_cost(source, i) == 0 {
                self.csr.push_flow(source, i, delta, &self.canonical_labels);
            }
        }

        for u in 0..self.csr.num_nodes {
            if u != source && u != sink && self.csr.excesses[u] > Flow::zero() {
                self.active_nodes.push_back(u);
            }
        }
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
        self.current_edge[u] = self.csr.start[u];

        // relabel
        if self.distance_count[self.distances[u]] == 1 {
            self.gap_relabeling(self.distances[u]);
        } else {
            self.relabel(u);
        }

        if self.csr.excesses[u] > Flow::zero() {
            self.active_nodes.push_back(u);
        }
    }

    // push from u
    fn push(&mut self, u: usize, i: usize) {
        let to = self.csr.to[i];
        let delta = self.csr.excesses[u].min(self.csr.residual_capacity(i));
        let is_zero = self.csr.excesses[to] == Flow::zero();
        if self.is_admissible(u, i) && delta > Flow::zero() {
            self.csr.push_flow(u, i, delta, &self.canonical_labels);
            if is_zero {
                self.active_nodes.push_back(to);
            }
        }
    }

    fn relabel(&mut self, u: usize) {
        self.relabel_count += 1;
        self.distance_count[self.distances[u]] -= 1;

        let new_distance = self
            .csr
            .neighbors(u)
            .filter(|&i| self.csr.residual_capacity(i) > Flow::zero() && self.reduced_cost(u, i) == 0)
            .map(|i| self.distances[self.csr.to[i]] + 1)
            .min()
            .unwrap()
            .min(self.csr.num_nodes);

        self.distances[u] = new_distance;
        self.distance_count[self.distances[u]] += 1;
    }

    fn update_distances(&mut self, source: usize, sink: usize) {
        self.distances.fill(self.csr.num_nodes);
        self.distances[sink] = 0;

        self.que.clear();
        self.que.push_back(sink);

        while let Some(v) = self.que.pop_front() {
            for i in self.csr.start[v]..self.csr.start[v + 1] {
                let to = self.csr.to[i];
                if self.csr.flow[i] > Flow::zero() && self.distances[to] == self.csr.num_nodes && self.reduced_cost_rev(v, i) == 0 {
                    self.distances[to] = self.distances[v] + 1;
                    if to != source {
                        self.que.push_back(to);
                    }
                }
            }
        }
    }

    // gap relabeling heuristic
    // set distance[u] >= k to distance[u] = n
    // O(n)
    fn gap_relabeling(&mut self, k: usize) {
        for u in 0..self.csr.num_nodes {
            if self.distances[u] >= k {
                self.distance_count[self.distances[u]] -= 1;
                self.distances[u] = self.distances[u].max(self.csr.num_nodes);
                self.distance_count[self.distances[u]] += 1;
            }
        }
    }

    fn push_flow_excess_back_to_source(&mut self, source: usize, sink: usize) {
        let mut visited = BitVector::new(self.csr.num_nodes);

        for u in 0..self.csr.num_nodes {
            if u == source || u == sink {
                continue;
            }
            while self.csr.excesses[u] > Flow::zero() {
                self.current_edge.iter_mut().enumerate().for_each(|(u, e)| *e = self.csr.start[u]);
                let alpha = self.dfs(u, source, self.labeled_excess(u), &mut visited);
                self.csr.excesses[u] = self.csr.excesses[u] - alpha * self.canonical_labels[u];
                self.csr.excesses[source] = self.csr.excesses[source] + alpha * self.canonical_labels[source];

                let eps = Flow::from(1e-10).unwrap();
                if self.csr.excesses[u] <= eps {
                    self.csr.excesses[u] = Flow::zero();
                }
                visited.clear();
            }
        }
    }

    fn dfs(&mut self, u: usize, source: usize, labeled_flow: Flow, visited: &mut BitVector) -> Flow {
        if u == source {
            return labeled_flow;
        }
        visited.set(u, true);

        for i in self.current_edge[u]..self.csr.start[u + 1] {
            self.current_edge[u] = i;
            let to = self.csr.to[i];
            let residual_capacity = self.labeled_residual_capacity(u, i);
            if visited.get(to) || residual_capacity == Flow::zero() {
                continue;
            }

            if self.reduced_cost(u, i) != 0 {
                continue;
            }

            let delta = self.dfs(to, source, labeled_flow.min(residual_capacity), visited);
            if delta > Flow::zero() {
                self.csr.push_labeled_flow(u, i, delta, &self.canonical_labels, false);
                return delta;
            }
        }
        Flow::zero()
    }

    #[inline]
    fn is_admissible(&self, u: usize, i: usize) -> bool {
        self.csr.residual_capacity(i) > Flow::zero() && self.distances[u] == self.distances[self.csr.to[i]] + 1 && self.reduced_cost(u, i) == 0
    }

    #[inline]
    fn labeled_excess(&self, u: usize) -> Flow {
        self.csr.excesses[u] / self.canonical_labels[u]
    }

    #[inline]
    fn labeled_residual_capacity(&self, u: usize, i: usize) -> Flow {
        self.csr.residual_capacity(i) / self.canonical_labels[u]
    }

    #[inline]
    fn reduced_cost(&self, u: usize, i: usize) -> Dist {
        self.csr.dist[i] - self.csr.potentials[u] + self.csr.potentials[self.csr.to[i]]
    }

    #[inline]
    fn reduced_cost_rev(&self, u: usize, i: usize) -> Dist {
        -self.csr.dist[i] + self.csr.potentials[u] - self.csr.potentials[self.csr.to[i]]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample() {
        let epsilon = 0.01;
        let mut graph = Graph::default();
        graph.add_nodes(8);

        graph.add_directed_edge(0, 1, 12.0, 0.7);
        graph.add_directed_edge(0, 2, 3.0, 0.9);
        graph.add_directed_edge(0, 3, 4.0, 0.8);

        graph.add_directed_edge(1, 4, 3.0, 0.5);
        graph.add_directed_edge(1, 5, 5.0, 0.8);

        graph.add_directed_edge(2, 1, 2.7, 1.0);
        graph.add_directed_edge(2, 3, 20.0 / 9.0, 0.9);
        graph.add_directed_edge(2, 5, 5.0, 0.7);

        graph.add_directed_edge(3, 5, 1.0, 1.0);
        graph.add_directed_edge(3, 6, 2.0, 0.7);

        graph.add_directed_edge(4, 7, 2.0, 0.5);

        graph.add_directed_edge(5, 4, 1.0, 0.5);
        graph.add_directed_edge(5, 6, 6.0, 0.7);
        graph.add_directed_edge(5, 7, 1.3, 1.0);

        graph.add_directed_edge(6, 7, 7.0, 1.0);

        PrimalDualPushRelabel::new(epsilon).solve(0, 7, &mut graph);

        let actual = graph.maximum_flow(7);

        let expected = 7.363;
        assert!(expected * (1.0 - epsilon) <= actual && actual <= expected, "{}/{}", actual, expected);
    }
}
