use crate::data_structures::skew_heap::SkewHeap;
use crate::data_structures::UnionFind;
use crate::edge::weight::WeightEdge;
use crate::prelude::{Directed, EdgeId, Graph};
use crate::traits::{Bounded, IntNum, Zero};
use std::marker::PhantomData;
use std::mem;

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
        let mut enter = vec![(usize::MAX, W::max_value(), EdgeId(usize::MAX)); num_nodes];
        let mut enter_edges = vec![SkewHeap::<W, Edge>::default(); num_nodes]; // in_edges[v] = all incoming edges of v
        let mut rset = Vec::new();
        let mut min: Vec<usize> = (0..num_nodes).collect();
        let mut cycles: Vec<Vec<EdgeId>> = vec![Vec::new(); num_nodes];

        // forest F
        let mut lambda = vec![usize::MAX; num_nodes];
        let mut parent = vec![usize::MAX; graph.num_edges()];
        let mut children = vec![Vec::new(); graph.num_edges()];
        let mut indegree = vec![0; graph.num_edges()];
        let mut aru = vec![false; graph.num_edges()];

        for (idx, edge) in graph.edges.iter().enumerate() {
            enter_edges[edge.v.index()].push(edge.data.weight, Edge { id: EdgeId(idx), from: edge.u.index() });
        }

        let mut roots: Vec<usize> = (0..num_nodes).collect(); // array of root components

        while let Some(k) = roots.pop() {
            let v = uf_scc.find(k);
            let (maximum_weight, edge) = enter_edges[v].pop().unwrap_or((W::zero(), Edge::default()));

            // no positive weight incoming edge of v
            if maximum_weight <= W::zero() {
                rset.push(v);
                continue;
            }

            let u = uf_scc.find(edge.from);

            // u and v are in the same scc
            if uf_scc.same(u, v) {
                roots.push(k);
                continue;
            }

            enter[v] = (u, maximum_weight, edge.id);
            aru[edge.id.index()] = true;

            if cycles[v].is_empty() {
                lambda[v] = edge.id.index();
            } else {
                while let Some(cycle_edge_id) = cycles[v].pop() {
                    parent[cycle_edge_id.index()] = edge.id.index();
                    children[edge.id.index()].push(cycle_edge_id.index());
                    indegree[cycle_edge_id.index()] += 1;
                }
            }

            // u and v are not in the same wcc
            if uf_wcc.union(u, v) {
                continue;
            }

            // contract cycle
            let mut cycle_edges = vec![edge.id];
            let mut cycle_nodes = vec![v];
            let mut minimum_weight_in_cycle = maximum_weight;
            let mut vertex = uf_scc.find(v);
            let mut now = uf_scc.find(u);
            loop {
                let (prev, weight, edge_id) = enter[now];
                cycle_nodes.push(now);
                cycle_edges.push(edge_id);

                if weight < minimum_weight_in_cycle {
                    minimum_weight_in_cycle = weight;
                    vertex = uf_scc.find(now);
                }

                let prev = uf_scc.find(prev);
                if prev == v {
                    break;
                }
                now = uf_scc.find(prev);
            }

            let mut scc = v;
            for &now in cycle_nodes.iter() {
                // adjust weight
                enter_edges[now].add_all(minimum_weight_in_cycle - enter[now].1);
                assert_ne!(enter[now].1, W::max_value());

                uf_scc.union(scc, now);
                uf_wcc.union(scc, now);
                let new_scc = uf_scc.find(scc);

                // move edges to new scc
                let other = mem::take(&mut enter_edges[now]);
                enter_edges[new_scc].merge_with(other);
                let other = mem::take(&mut enter_edges[scc]);
                enter_edges[new_scc].merge_with(other);

                scc = new_scc;
            }
            cycles[scc] = cycle_edges;
            min[scc] = min[vertex];
            roots.push(scc);
        }

        let mut delete = vec![false; graph.num_edges()];
        for root in rset {
            let root = min[root];
            let mut edge_id = lambda[root];
            while edge_id != usize::MAX {
                delete[edge_id] = true;
                for &c in children[edge_id].iter() {
                    parent[c] = usize::MAX;
                    indegree[c] -= 1;
                }
                edge_id = parent[edge_id];
            }
        }

        // construct branching
        let mut stack = Vec::new();
        for edge_id in 0..graph.num_edges() {
            if indegree[edge_id] == 0 && !delete[edge_id] && aru[edge_id] {
                stack.push(edge_id);
            }
        }

        let mut total_cost = W::zero();
        let mut branchings = Vec::with_capacity(num_nodes - 1);

        while let Some(edge_id) = stack.pop() {
            assert!(!delete[edge_id]);
            branchings.push(EdgeId(edge_id));
            total_cost += graph.edges[edge_id].data.weight;

            let v = graph.edges[edge_id].v.index();
            let mut edge_id = lambda[v];

            while edge_id != usize::MAX {
                delete[edge_id] = true;
                for &c in children[edge_id].iter() {
                    parent[c] = usize::MAX;
                    indegree[c] -= 1;
                    if indegree[c] == 0 && !delete[c] {
                        stack.push(c);
                    }
                }
                edge_id = parent[edge_id];
            }
        }

        (total_cost, branchings)
    }
}

mod tests {
    use super::*;

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
    fn test8() {
        let mut g: Graph<Directed, (), WeightEdge<i32>> = Graph::new_directed();
        let nodes = g.add_nodes(4);
        g.add_directed_edge(nodes[0], nodes[1], 18);
        g.add_directed_edge(nodes[0], nodes[3], 13);
        g.add_directed_edge(nodes[1], nodes[2], 15);
        g.add_directed_edge(nodes[2], nodes[0], 4);
        g.add_directed_edge(nodes[3], nodes[2], 13);

        let mut solver = Tarjan::default();
        // let mut solver = Edmonds::default();
        let (cost, arborescence) = solver.solve(&g);
        let mut used = vec![false; g.num_nodes()];

        println!("graph");
        for &edge_id in arborescence.iter() {
            println!("{:?}", g.get_edge(edge_id));
        }

        for edge_id in arborescence {
            assert!(!used[g.get_edge(edge_id).v.index()]);
            used[g.get_edge(edge_id).v.index()] = true;
        }
        assert_eq!(cost, 46);
    }

    #[test]
    fn test9() {
        let mut g: Graph<Directed, (), WeightEdge<i32>> = Graph::new_directed();
        let nodes = g.add_nodes(5);
        g.add_directed_edge(nodes[1], nodes[4], 10);
        g.add_directed_edge(nodes[2], nodes[0], 2);
        g.add_directed_edge(nodes[2], nodes[1], 6);
        g.add_directed_edge(nodes[3], nodes[4], 9);
        g.add_directed_edge(nodes[4], nodes[2], 4);

        let mut solver = Tarjan::default();
        // let mut solver = Edmonds::default();
        let (cost, arborescence) = solver.solve(&g);
        let mut used = vec![false; g.num_nodes()];

        println!("graph");
        for &edge_id in arborescence.iter() {
            println!("{:?}", g.get_edge(edge_id));
        }

        for edge_id in arborescence {
            assert!(!used[g.get_edge(edge_id).v.index()]);
            used[g.get_edge(edge_id).v.index()] = true;
        }
        assert_eq!(cost, 21);
    }

    #[test]
    fn test10() {
        let mut g: Graph<Directed, (), WeightEdge<i32>> = Graph::new_directed();
        let nodes = g.add_nodes(5);
        g.add_directed_edge(nodes[0], nodes[1], 2);
        g.add_directed_edge(nodes[1], nodes[4], 8);
        g.add_directed_edge(nodes[3], nodes[4], 8);
        g.add_directed_edge(nodes[4], nodes[0], 9);
        g.add_directed_edge(nodes[4], nodes[3], 5);

        let mut solver = Tarjan::default();
        // let mut solver = Edmonds::default();
        let (cost, arborescence) = solver.solve(&g);
        let mut used = vec![false; g.num_nodes()];

        println!("graph");
        for &edge_id in arborescence.iter() {
            println!("{:?}", g.get_edge(edge_id));
        }

        for edge_id in arborescence {
            assert!(!used[g.get_edge(edge_id).v.index()]);
            used[g.get_edge(edge_id).v.index()] = true;
        }
        assert_eq!(cost, 22);
    }
}
