use crate::algorithms::generalized_maximum_flow::csr::{Dist, CSR};
use crate::algorithms::generalized_maximum_flow::generalized_maximum_flow_graph::Graph;
use crate::algorithms::generalized_maximum_flow::status::Status;
use num_traits::{Float, FromPrimitive, ToPrimitive};
use std::collections::VecDeque;

#[derive(Default)]
pub struct PrimalDual<Flow> {
    csr: CSR<Flow>,
    canonical_labels: Vec<Flow>,

    // maximum flow(dinic)
    current_edge: Box<[usize]>,
    distances: Box<[usize]>,
    que: VecDeque<usize>,
}

#[allow(dead_code)]
impl<Flow> PrimalDual<Flow>
where
    Flow: Float + PartialOrd + Copy + Clone + ToPrimitive + FromPrimitive + Default,
{
    pub fn new(epsilon: Flow) -> Self {
        Self { csr: CSR::new(epsilon), ..Default::default() }
    }

    pub fn solve(&mut self, source: usize, sink: usize, graph: &mut Graph<Flow>) -> Status {
        self.csr.build(graph);
        self.canonical_labels = vec![Flow::zero(); self.csr.num_nodes];
        self.current_edge = vec![0; self.csr.num_nodes].into_boxed_slice();
        self.distances = vec![0; self.csr.num_nodes].into_boxed_slice();

        self.csr.excesses[source] = Flow::max_value();
        loop {
            let distance_to_sink = self.csr.calculate_distance_to_sink(sink);
            self.update_canonical_labels(&distance_to_sink, sink);

            // no augmenting path from source
            if self.canonical_labels[source] == Flow::max_value() {
                break;
            }

            // maximum flow
            while self.csr.excesses[source] > Flow::epsilon() {
                self.bfs(source, sink);

                // no s-t path
                if self.distances[source] >= self.csr.num_nodes {
                    break;
                }

                self.current_edge.iter_mut().enumerate().for_each(|(u, e)| *e = self.csr.start[u]);
                match self.dfs(source, sink, self.csr.excesses[source] / self.canonical_labels[source]) {
                    Some(alfa) => self.csr.excesses[source] = self.csr.excesses[source] - alfa * self.canonical_labels[source],
                    None => break,
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

    fn bfs(&mut self, source: usize, sink: usize) {
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

    fn dfs(&mut self, u: usize, sink: usize, upper: Flow) -> Option<Flow> {
        if u == sink {
            return Some(upper);
        }

        let mut res = Flow::zero();
        for i in self.current_edge[u]..self.csr.start[u + 1] {
            self.current_edge[u] = i;

            let to = self.csr.to[i];
            if self.is_admissible(u, i) {
                if let Some(delta) = self.dfs(to, sink, upper.min(self.labeled_residual_capacity(u, i))) {
                    self.csr.push_labeled_flow(u, i, delta, &self.canonical_labels, false);
                    res = res + delta;
                    if res == upper {
                        return Some(res);
                    }
                }
            }
        }
        self.current_edge[u] = self.csr.start[u + 1];
        self.distances[u] = self.csr.num_nodes;

        Some(res)
    }

    #[inline]
    fn is_admissible(&self, u: usize, i: usize) -> bool {
        self.csr.residual_capacity(i) > Flow::zero() && self.distances[u] == self.distances[self.csr.to[i]] + 1 && self.reduced_cost(u, i) == 0
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

        PrimalDual::new(epsilon).solve(0, 7, &mut graph);

        let actual = graph.maximum_flow(7);

        let expected = 7.363;
        assert!(expected * (1.0 - epsilon) <= actual && actual <= expected, "{}/{}", actual, expected);
    }
}
