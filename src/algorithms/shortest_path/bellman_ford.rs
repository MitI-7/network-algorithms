use crate::algorithms::shortest_path::csr::CSR;
use crate::algorithms::shortest_path::edge::WeightEdge;
use crate::core::numeric::FlowNum;
use crate::graph::direction::Directed;
use crate::graph::graph::Graph;
use crate::graph::ids::NodeId;

#[derive(Default)]
pub struct BellmanFord<W> {
    csr: CSR<W>,
}

impl<W> BellmanFord<W>
where
    W: FlowNum,
{
    pub fn solve(
        &mut self,
        graph: &Graph<Directed, (), WeightEdge<W>>,
        source: NodeId,
    ) -> Result<Vec<Option<W>>, String> {
        self.csr.build(graph);

        let mut distances = vec![None; self.csr.num_nodes];
        distances[source.index()] = Some(W::zero());

        let mut num_loop = 0;
        loop {
            let mut update = false;
            for u in 0..self.csr.num_nodes {
                if distances[u].is_none() {
                    continue;
                }

                for i in self.csr.neighbors(u) {
                    let to = self.csr.to[i];
                    let w = self.csr.weight[i];
                    if distances[to].is_none()
                        || (distances[u].is_some() && distances[to].unwrap() > distances[u].unwrap() + w)
                    {
                        distances[to] = Some(distances[u].unwrap() + w);
                        update = true;
                    }
                }
            }
            if !update {
                break;
            }
            num_loop += 1;
        }

        // cycle found
        if num_loop == self.csr.num_nodes {
            Err("cycle found".to_string())
        } else {
            Ok(distances)
        }
    }
}
//
// #[cfg(test)]
// mod test {
//     use crate::solvers::shortest_path::bellman_ford::BellmanFord;
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
//         let distances = BellmanFord::default().solve(&graph, 0).unwrap();
//
//         assert_eq!(distances, [Some(0), Some(1), Some(3), Some(4)]);
//     }
// }
