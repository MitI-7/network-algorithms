use crate::algorithms::generalized_maximum_flow::csr::CSR;
use crate::algorithms::generalized_maximum_flow::generalized_maximum_flow_graph::Graph;
use crate::algorithms::generalized_maximum_flow::status::Status;
use crate::algorithms::generalized_maximum_flow::GeneralizedMaximumFlowNum;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

pub struct HighestGainPath<Flow> {
    csr: CSR<Flow>,
    excesses: Vec<Flow>,
}

#[allow(dead_code)]
impl<Flow> HighestGainPath<Flow>
where
    Flow: GeneralizedMaximumFlowNum,
{
    pub fn new(epsilon: Flow) -> Self {
        HighestGainPath { csr: CSR::new(epsilon), excesses: Vec::new() }
    }

    pub fn solve(&mut self, source: usize, sink: usize, graph: &mut Graph<Flow>) -> Status {
        self.csr.build(graph);
        self.excesses.resize(graph.num_nodes(), Flow::zero());

        self.excesses[source] = Flow::max_value();
        while self.excesses[source] > Flow::epsilon() {
            if !self.argument_flow(source, sink) {
                break;
            }
        }

        // copy
        for u in 0..graph.num_nodes() {
            graph.excesses[u] = self.csr.excesses[u];
        }
        for edge_id in 0..graph.num_edges() {
            let i = self.csr.edge_index_to_inside_edge_index[edge_id];
            graph.edges[edge_id].flow = self.csr.flow[i];
        }

        Status::Optimal
    }

    fn argument_flow(&mut self, source: usize, sink: usize) -> bool {
        match self.find_shortest_path(source, sink) {
            None => false,
            Some(prev) => {
                // calculate delta and canonical labels
                let mut delta = Flow::max_value();
                let mut canonical_labels = vec![Flow::max_value(); self.csr.num_nodes];
                {
                    canonical_labels[sink] = Flow::one();

                    let mut dist_to_sink = 0;
                    let mut v = sink;
                    while v != source {
                        // u -> v
                        let (u, edge_index) = prev[v];
                        dist_to_sink += self.csr.dist[edge_index];
                        let a = Flow::from(dist_to_sink).unwrap();
                        canonical_labels[u] = self.csr.base.powf(a);

                        delta = delta.min(self.labeled_residual_capacity(u, edge_index, &canonical_labels));
                        v = u;
                    }

                    delta = delta.min(self.excesses[source] / canonical_labels[source]);
                }

                // update flow
                let mut v = sink;
                while v != source {
                    // u -> v
                    let (u, i) = prev[v];
                    self.csr.push_labeled_flow(u, i, delta, &canonical_labels, false);
                    v = u;
                }

                self.excesses[source] = self.excesses[source] - canonical_labels[source] * delta;
                self.excesses[sink] = self.excesses[sink] + delta;

                true
            }
        }
    }

    // find the shortest path from source to sink & update potentials
    pub fn find_shortest_path(&mut self, source: usize, sink: usize) -> Option<Vec<(usize, usize)>> {
        let mut prev = vec![(self.csr.num_nodes, self.csr.num_nodes); self.csr.num_nodes];

        let mut visited = vec![false; self.csr.num_nodes];
        let mut distance = vec![None; self.csr.num_nodes];

        distance[source] = Some(0);

        let mut heap = BinaryHeap::new();
        heap.push((Reverse(0), source));
        while let Some((d, u)) = heap.pop() {
            if visited[u] {
                continue;
            }
            visited[u] = true;
            if u == sink {
                break;
            }

            for edge_index in self.csr.neighbors(u) {
                if self.csr.residual_capacity(edge_index) < Flow::epsilon() {
                    continue;
                }

                let to = self.csr.to[edge_index];
                if visited[to] {
                    continue;
                }

                let ed = self.csr.dist[edge_index];
                let dist = ed + self.csr.potentials[u] - self.csr.potentials[to];
                assert!(dist >= 0, "{}, {}, {}, {}", dist, ed, self.csr.potentials[u], self.csr.potentials[to]);

                let new_dist = d.0 + dist;
                if distance[to].is_none() || new_dist < distance[to].unwrap() {
                    distance[to] = Some(new_dist);
                    prev[to] = (u, edge_index);
                    heap.push((Reverse(new_dist), to));
                }
            }
        }

        if !visited[sink] {
            return None;
        }

        // update potentials
        for u in 0..self.csr.num_nodes {
            if visited[u] {
                self.csr.potentials[u] += distance[u].unwrap() - distance[sink].unwrap();
            }
        }

        Some(prev)
    }

    #[inline]
    fn labeled_residual_capacity(&self, u: usize, i: usize, labels: &[Flow]) -> Flow {
        self.csr.residual_capacity(i) / labels[u]
    }
}
// 
// #[cfg(test)]
// mod tests {
//     use super::*;
// 
//     #[test]
//     fn sample() {
//         let epsilon = 0.01;
//         let mut graph = Graph::default();
//         graph.add_nodes(8);
// 
//         graph.add_directed_edge(0, 1, 12.0, 0.7);
//         graph.add_directed_edge(0, 2, 3.0, 0.9);
//         graph.add_directed_edge(0, 3, 4.0, 0.8);
// 
//         graph.add_directed_edge(1, 4, 3.0, 0.5);
//         graph.add_directed_edge(1, 5, 5.0, 0.8);
// 
//         graph.add_directed_edge(2, 1, 2.7, 1.0);
//         graph.add_directed_edge(2, 3, 20.0 / 9.0, 0.9);
//         graph.add_directed_edge(2, 5, 5.0, 0.7);
// 
//         graph.add_directed_edge(3, 5, 1.0, 1.0);
//         graph.add_directed_edge(3, 6, 2.0, 0.7);
// 
//         graph.add_directed_edge(4, 7, 2.0, 0.5);
// 
//         graph.add_directed_edge(5, 4, 1.0, 0.5);
//         graph.add_directed_edge(5, 6, 6.0, 0.7);
//         graph.add_directed_edge(5, 7, 1.3, 1.0);
// 
//         graph.add_directed_edge(6, 7, 7.0, 1.0);
// 
//         HighestGainPath::new(epsilon).solve(0, 7, &mut graph);
// 
//         let actual = graph.maximum_flow(7);
// 
//         let expected = 7.363;
//         assert!(expected * (1.0 - epsilon) <= actual && actual <= expected, "{}/{}", actual, expected);
//     }
// }
