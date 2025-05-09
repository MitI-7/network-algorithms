use crate::core::graph::Graph;
use crate::data_structures::bit_vector;
use crate::traits::{IntNum, Zero};
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use crate::core::direction::Directed;
use crate::edge::weight::WeightEdge;
use crate::algorithms::shortest_path::csr::CSR;

#[derive(Default)]
struct Dijkstra<W> {
    csr: CSR<W>,
}

impl<W> Dijkstra<W>
where
    W: IntNum + Zero + Copy + Ord,
{
    fn solve<E>(&mut self, graph: &Graph<Directed, (), WeightEdge<W>>, source: usize) -> Vec<W> {
        self.csr.build(graph);

        let mut heap = BinaryHeap::new();
        heap.push((Reverse(W::zero()), source));

        let mut visited = bit_vector::BitVector::new(self.csr.num_nodes);
        let mut distance = vec![W::max_value(); self.csr.num_nodes];
        let mut prev = vec![usize::MAX; self.csr.num_nodes];
        distance[source] = W::zero();

        while let Some((d, u)) = heap.pop() {
            if visited.get(u) {
                continue;
            }
            visited.set(u, true);

            for i in self.csr.neighbors(u) {
                let to = self.csr.to[i];
                let w = self.csr.weight[i];

                if visited.get(to) {
                    continue;
                }

                let new_dist = d.0 + w;
                if new_dist < distance[to] {
                    distance[to] = new_dist;
                    prev[to] = u;
                    heap.push((Reverse(new_dist), to));
                }
            }
        }

        distance
        // for u in 0..self.graph.num_nodes() {
        //     if visited.access(u) {
        //         self.distance[u] = Some(distance[u]);
        //     }
        //     // println!("{:?}", distance[u]);
        // }

        // if !visited.access(t) {
        //     println!("{}", -1);
        //     return;
        // }
        //
        // let mut route = Vec::new();
        // {
        //     let mut v = t;
        //     while prev[v] != usize::MAX {
        //         let u = prev[v];
        //         route.push((u, v));
        //         v = u;
        //     }
        // }
        //
        // let stdout = std::io::stdout();

        // let mut out = BufWriter::new(stdout.lock());
        // writeln!(out, "{} {}", distance[t], route.len()).unwrap();
        // for (u, v) in route.iter().rev() {
        //     writeln!(out, "{} {}", u, v).unwrap();
        // }
    }
}

// #[cfg(test)]
// mod test {
//     use crate::algorithms::shortest_path::dijkstra::Dijkstra;
//     use crate::graph::edge::WeightEdge;
//     use crate::graph::graph::Directed;
//     use crate::graph::Graph;
//
//     // https://ja.wikipedia.org/wiki/%E6%9C%80%E5%A4%A7%E3%83%95%E3%83%AD%E3%83%BC%E5%95%8F%E9%A1%8C
//     #[test]
//     fn test_max_flow_wikipedia() {
//         let mut graph: Graph<WeightEdge<i32>, Directed> = Graph::new_directed();
//         graph.add_nodes(4);
//         graph.add_edge(0, 1, WeightEdge { weight: 1 }).unwrap();
//         graph.add_edge(0, 2, WeightEdge { weight: 4 }).unwrap();
//         graph.add_edge(1, 2, WeightEdge { weight: 2 }).unwrap();
//         graph.add_edge(2, 3, WeightEdge { weight: 1 }).unwrap();
//         graph.add_edge(1, 3, WeightEdge { weight: 5 }).unwrap();
//
//         let distances = Dijkstra::default().solve(&graph, 0);
//
//         assert_eq!(distances, [0, 1, 3, 4]);
//     }
// }
