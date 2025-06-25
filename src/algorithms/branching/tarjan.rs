use crate::data_structures::skew_heap::SkewHeap;
use crate::data_structures::UnionFind;
use crate::edge::weight::WeightEdge;
use crate::prelude::{Directed, EdgeId, Graph};
use crate::traits::{Bounded, IntNum, Zero};
use std::collections::HashSet;
use std::marker::PhantomData;
use std::mem;
use std::ops::Index;

#[derive(Default)]
pub struct Tarjan<W> {
    phantom_data: PhantomData<W>,
}

impl<W> Tarjan<W>
where
    W: IntNum + Zero + Bounded + std::ops::Neg<Output = W> + Default + std::fmt::Debug + std::fmt::Display,
{
    pub fn solve(&mut self, graph: &Graph<Directed, (), WeightEdge<W>>) -> (W, Vec<EdgeId>) {
        #[derive(Clone, Debug, Default)]
        struct Edge {
            id: EdgeId,
            from: usize,
        }

        let num_nodes = graph.num_nodes();

        let mut uf_wcc = UnionFind::new(num_nodes);
        let mut uf_scc = UnionFind::new(num_nodes);
        let mut parent = vec![(usize::MAX, W::max_value()); num_nodes];
        let mut in_edges = vec![SkewHeap::<W, Edge>::default(); num_nodes];
        let mut rset = Vec::new();
        let mut min: Vec<usize> = (0..num_nodes).collect();
        let mut h = vec![Vec::new(); num_nodes];

        for (idx, edge) in graph.edges.iter().enumerate() {
            in_edges[edge.v.index()].push(edge.data.weight, Edge { id: EdgeId(idx), from: edge.u.index() });
        }

        let mut roots: Vec<usize> = (0..num_nodes).collect();
        roots.reverse();

        while let Some(k) = roots.pop() {
            let v = uf_scc.leader(k);

            let (maximum_weight, edge) = in_edges[v].pop().unwrap_or((W::zero(), Edge::default()));
            // No positive-weight incoming edge to v
            if maximum_weight <= W::zero() {
                rset.push(v);
                continue;
            }

            let u = uf_scc.leader(edge.from);

            // u and v are in the same scc
            if uf_scc.same_set(u, v) {
                roots.push(k);
                continue;
            }

            h[edge.from].push(edge.id);

            // u and v are not in the same wcc
            if uf_wcc.unite(u, v) {
                parent[v] = (u, maximum_weight);
                assert_ne!(parent[v].0, v);
                continue;
            }

            // contract cycle
            // println!("cycle");
            {
                let mut nodes = Vec::new();
                let mut minimum_weight_in_cycle = maximum_weight;
                let mut vertex = uf_scc.leader(v);
                let mut cur = uf_scc.leader(u);
                loop {
                    nodes.push(cur);

                    let (par, w) = parent[cur];
                    let par = uf_scc.leader(par);
                    if w < minimum_weight_in_cycle {
                        minimum_weight_in_cycle = w;
                        vertex = uf_scc.leader(cur);
                    }
                    if par == v {
                        break;
                    }
                    cur = uf_scc.leader(par);
                }

                assert_eq!(
                    nodes.len(),
                    {
                        let mut hs: HashSet<usize> = HashSet::new();
                        hs.extend(nodes.iter());
                        hs.len()
                    },
                    "nodes に重複頂点が含まれています"
                );

                // adjust weight
                in_edges[v].add_all(minimum_weight_in_cycle - maximum_weight);
                for &w in nodes.iter() {
                    assert_ne!(parent[w].1, W::max_value());
                    in_edges[w].add_all(minimum_weight_in_cycle - parent[w].1);
                }

                // construct
                let mut scc = v;
                for &u in nodes.iter() {
                    uf_scc.unite(u, scc);
                    uf_wcc.unite(u, scc);
                    let a = uf_scc.leader(scc);
                    let b = u ^ scc ^ a;
                    if b != a {
                        let other = mem::take(&mut in_edges[b]);
                        in_edges[a].merge_with(other);
                    }
                    scc = a;
                }

                min[scc] = min[vertex];

                for &u in nodes.iter() {
                    parent[u] = (usize::MAX, W::max_value());
                }
                parent[v] = (usize::MAX, W::max_value());

                roots.push(scc);
            }
        }

        let mut total_cost = W::zero();
        let mut arborescence = Vec::with_capacity(num_nodes - 1);
        let mut visited = vec![false; num_nodes];
        let mut s = HashSet::new();
        for r in rset {
            s.insert(min[r]);
        }
        for r in s {
            let mut stack = vec![r];
            while let Some(u) = stack.pop() {
                visited[u] = true;
                for edge_id in h[u].iter() {
                    let edge = &graph.edges[edge_id.index()];
                    if !visited[edge.v.index()] {
                        visited[edge.v.index()] = true;
                        arborescence.push(*edge_id);
                        total_cost += edge.data.weight;
                        stack.push(edge.v.index());
                    }
                }
            }
            // use std::collections::VecDeque;
            // let mut queue = VecDeque::from(vec![r]);
            // while let Some(u) = queue.pop_front() {           // FIFO
            //     visited[u] = true;
            //     for &edge_id in &h[u] {
            //         let edge = &graph.edges[edge_id.index()];
            //         let v = edge.v.index();
            //         if !visited[v] {
            //             visited[v] = true;
            //             arborescence.push(edge_id);
            //             total_cost += edge.data.weight;
            //             queue.push_back(v);                   // 次を末尾に
            //         }
            //     }
            // }
        }

        (total_cost, arborescence)
    }
}

mod tests {
    use super::*;
    use crate::algorithms::branching::Edmonds;

    #[test]
    fn test1() {
        let mut g: Graph<Directed, (), WeightEdge<i32>> = Graph::new_directed();
        let nodes = g.add_nodes(4);
        g.add_directed_edge(nodes[0], nodes[1], 10);
        g.add_directed_edge(nodes[1], nodes[2], 9);
        g.add_directed_edge(nodes[2], nodes[0], 8);
        g.add_directed_edge(nodes[3], nodes[1], 7);

        let mut solver = Tarjan::default();
        let (cost, arborescence) = solver.solve(&g);
        for edge_id in arborescence {
            println!("{:?}", g.get_edge(edge_id));
        }
        assert_eq!(cost, 24);
    }

    #[test]
    fn test2() {
        let mut g: Graph<Directed, (), WeightEdge<i32>> = Graph::new_directed();
        let nodes = g.add_nodes(4);
        g.add_directed_edge(nodes[0], nodes[1], 3);
        g.add_directed_edge(nodes[0], nodes[2], 2);
        g.add_directed_edge(nodes[2], nodes[0], 1);
        g.add_directed_edge(nodes[2], nodes[3], 1);
        g.add_directed_edge(nodes[3], nodes[0], 1);
        g.add_directed_edge(nodes[3], nodes[1], 5);

        let mut solver = Tarjan::default();
        let (cost, arborescence) = solver.solve(&g);
        for edge_id in arborescence {
            println!("{:?}", g.get_edge(edge_id));
        }
        assert_eq!(cost, 8);
    }

    #[test]
    fn test3() {
        let mut g: Graph<Directed, (), WeightEdge<i32>> = Graph::new_directed();
        let nodes = g.add_nodes(6);
        g.add_directed_edge(nodes[0], nodes[2], 7);
        g.add_directed_edge(nodes[0], nodes[1], 1);
        g.add_directed_edge(nodes[0], nodes[3], 5);
        g.add_directed_edge(nodes[1], nodes[4], 9);
        g.add_directed_edge(nodes[2], nodes[1], 6);
        g.add_directed_edge(nodes[1], nodes[3], 2);
        g.add_directed_edge(nodes[3], nodes[4], 3);
        g.add_directed_edge(nodes[4], nodes[2], 2);
        g.add_directed_edge(nodes[2], nodes[5], 8);
        g.add_directed_edge(nodes[3], nodes[5], 3);

        let mut solver = Tarjan::default();
        let (cost, arborescence) = solver.solve(&g);
        for edge_id in arborescence {
            println!("{:?}", g.get_edge(edge_id));
        }
        assert_eq!(cost, 35);
    }

    #[test]
    fn test4() {
        let mut g: Graph<Directed, (), WeightEdge<i32>> = Graph::new_directed();
        let nodes = g.add_nodes(10);
        g.add_directed_edge(nodes[9], nodes[0], 7227);
        g.add_directed_edge(nodes[0], nodes[3], 1292);
        g.add_directed_edge(nodes[3], nodes[5], 2718);
        g.add_directed_edge(nodes[5], nodes[8], 7842);
        g.add_directed_edge(nodes[8], nodes[1], 7668);
        g.add_directed_edge(nodes[5], nodes[7], 453);
        g.add_directed_edge(nodes[5], nodes[2], 2870);
        g.add_directed_edge(nodes[6], nodes[7], 2643);
        g.add_directed_edge(nodes[4], nodes[0], 1649);
        g.add_directed_edge(nodes[7], nodes[0], 2818);
        g.add_directed_edge(nodes[6], nodes[0], 6617);
        g.add_directed_edge(nodes[9], nodes[4], 4584);
        g.add_directed_edge(nodes[6], nodes[5], 7242);
        g.add_directed_edge(nodes[2], nodes[6], 1267);
        g.add_directed_edge(nodes[2], nodes[7], 4877);

        let mut solver = Tarjan::default();
        let (cost, arborescence) = solver.solve(&g);
        let mut used = vec![false; g.num_nodes()];
        for edge_id in arborescence {
            println!("{:?}", g.get_edge(edge_id));
            assert!(!used[g.get_edge(edge_id).v.index()]);
            used[g.get_edge(edge_id).v.index()] = true;
        }
        assert_eq!(cost, 43602);
    }

    #[test]
    fn test5() {
        let mut g: Graph<Directed, (), WeightEdge<i32>> = Graph::new_directed();
        let nodes = g.add_nodes(4);
        g.add_directed_edge(nodes[0], nodes[1], 10);
        g.add_directed_edge(nodes[1], nodes[2], 20);
        g.add_directed_edge(nodes[2], nodes[0], 30);
        g.add_directed_edge(nodes[2], nodes[3], 100);

        let mut solver = Tarjan::default();
        let (cost, arborescence) = solver.solve(&g);
        let mut used = vec![false; g.num_nodes()];
        for edge_id in arborescence {
            println!("{:?}", g.get_edge(edge_id));
            assert!(!used[g.get_edge(edge_id).v.index()]);
            used[g.get_edge(edge_id).v.index()] = true;
        }
        assert_eq!(cost, 150);
    }

    #[test]
    fn test6() {
        let mut g: Graph<Directed, (), WeightEdge<i32>> = Graph::new_directed();
        let nodes = g.add_nodes(10);
        g.add_directed_edge(nodes[0], nodes[2], 1175);
        g.add_directed_edge(nodes[2], nodes[1], 6460);
        g.add_directed_edge(nodes[1], nodes[4], 4761);
        g.add_directed_edge(nodes[4], nodes[8], 8294);
        g.add_directed_edge(nodes[8], nodes[5], 5954);
        g.add_directed_edge(nodes[5], nodes[9], 4564);
        g.add_directed_edge(nodes[9], nodes[7], 7208);
        g.add_directed_edge(nodes[5], nodes[3], 6028);
        g.add_directed_edge(nodes[1], nodes[0], 3092);
        g.add_directed_edge(nodes[5], nodes[0], 6527);
        g.add_directed_edge(nodes[7], nodes[5], 823);
        g.add_directed_edge(nodes[0], nodes[8], 8252);
        g.add_directed_edge(nodes[6], nodes[7], 1343);
        g.add_directed_edge(nodes[3], nodes[2], 8365);
        g.add_directed_edge(nodes[9], nodes[6], 4996);

        let mut solver = Tarjan::default();
        let (cost, arborescence) = solver.solve(&g);
        let mut used = vec![false; g.num_nodes()];
        for edge_id in arborescence {
            println!("{:?}", g.get_edge(edge_id));
            assert!(!used[g.get_edge(edge_id).v.index()]);
            used[g.get_edge(edge_id).v.index()] = true;
        }
        assert_eq!(cost, 58396);
    }

    #[test]
    fn test7() {
        let mut g: Graph<Directed, (), WeightEdge<i32>> = Graph::new_directed();
        let nodes = g.add_nodes(10);
        g.add_directed_edge(nodes[5], nodes[8], 3993);
        g.add_directed_edge(nodes[8], nodes[3], 4447);
        g.add_directed_edge(nodes[3], nodes[9], 6058);
        g.add_directed_edge(nodes[9], nodes[4], 8231);
        g.add_directed_edge(nodes[4], nodes[7], 7864);
        g.add_directed_edge(nodes[7], nodes[2], 9838);
        g.add_directed_edge(nodes[1], nodes[0], 4310);
        g.add_directed_edge(nodes[6], nodes[9], 8824);
        g.add_directed_edge(nodes[4], nodes[0], 2157);
        g.add_directed_edge(nodes[2], nodes[0], 1910);
        g.add_directed_edge(nodes[5], nodes[1], 3118);
        g.add_directed_edge(nodes[2], nodes[6], 7772);
        g.add_directed_edge(nodes[9], nodes[1], 4795);
        g.add_directed_edge(nodes[1], nodes[8], 8995);
        g.add_directed_edge(nodes[9], nodes[8], 3411);

        let mut solver = Tarjan::default();
        let (cost, arborescence) = solver.solve(&g);
        let mut used = vec![false; g.num_nodes()];
        for edge_id in arborescence {
            println!("{:?}", g.get_edge(edge_id));
            assert!(!used[g.get_edge(edge_id).v.index()]);
            used[g.get_edge(edge_id).v.index()] = true;
        }
        assert_eq!(cost, 60633);
    }

    #[test]
    fn test8() {
        let mut g: Graph<Directed, (), WeightEdge<i32>> = Graph::new_directed();
        let nodes = g.add_nodes(4);
        g.add_directed_edge(nodes[0], nodes[1], 18);
        g.add_directed_edge(nodes[0], nodes[3], 13);
        g.add_directed_edge(nodes[1], nodes[2], 15);
        g.add_directed_edge(nodes[2], nodes[0], 4);
        g.add_directed_edge(nodes[3], nodes[1], 8);
        g.add_directed_edge(nodes[3], nodes[2], 13);


        let mut solver = Tarjan::default();
        // let mut solver = Edmonds::default();
        let (cost, arborescence) = solver.solve(&g);
        let mut used = vec![false; g.num_nodes()];
        for edge_id in arborescence {
            println!("{:?}", g.get_edge(edge_id));
            assert!(!used[g.get_edge(edge_id).v.index()]);
            used[g.get_edge(edge_id).v.index()] = true;
        }
        assert_eq!(cost, 46);
    }
}
