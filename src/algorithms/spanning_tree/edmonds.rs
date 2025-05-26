use crate::core::direction::Directed;
use crate::core::graph::Graph;
use crate::edge::weight::WeightEdge;
use crate::prelude::EdgeId;
use crate::traits::{IntNum, Zero};
use std::marker::PhantomData;

#[derive(Clone)]
struct Edge<W> {
    id: EdgeId,
    from: usize,
    to: usize,
    cost: W,
}

// O(nm)
#[derive(Default)]
pub struct Edmonds<W> {
    phantom_data: PhantomData<W>,
}

impl<W> Edmonds<W>
where
    W: IntNum + Zero,
{
    pub fn solve(&self, graph: &Graph<Directed, (), WeightEdge<W>>, root: usize) -> Option<(W, Vec<EdgeId>)> {
        let mut edges = Vec::with_capacity(graph.num_edges());
        for (i, edge) in graph.edges.iter().enumerate() {
            edges.push(Edge { id: EdgeId(i), from: edge.u.index(), to: edge.v.index(), cost: edge.data.weight });
        }

        self.minimum_cost(graph.num_nodes(), &edges, root)
    }

    fn minimum_cost(&self, num_nodes: usize, edges: &Vec<Edge<W>>, root: usize) -> Option<(W, Vec<EdgeId>)> {
        let inf = W::max_value();
        let mut total_cost = W::zero();

        let mut in_cost = vec![inf; num_nodes];
        let mut parent = vec![usize::MAX; num_nodes];
        for &Edge { id, from, to, cost } in edges.iter() {
            if from != to && cost < in_cost[to] {
                in_cost[to] = cost;
                parent[to] = from;
            }
        }
        in_cost[root] = W::zero();
        for &c in in_cost.iter() {
            if c == inf {
                return None;
            }
            total_cost += c;
        }

        // decomposition of strongly connected components
        let mut ids = vec![usize::MAX; num_nodes];
        let mut scc_cnt = 0;
        {
            let mut stamp = vec![usize::MAX; num_nodes];
            for u in 0..num_nodes {
                let mut v = u;
                while stamp[v] != u && ids[v] == usize::MAX && v != root {
                    stamp[v] = u;
                    v = parent[v];
                }
                // find cycle
                if v != root && ids[v] == usize::MAX {
                    let mut w = parent[v];
                    ids[v] = scc_cnt;
                    while w != v {
                        ids[w] = scc_cnt;
                        w = parent[w];
                    }
                    scc_cnt += 1;
                }
            }

            // no cycle
            if scc_cnt == 0 {
                return Some((total_cost, Vec::new()));
            }

            for u in 0..num_nodes {
                if ids[u] == usize::MAX {
                    ids[u] = scc_cnt;
                    scc_cnt += 1;
                }
            }
        }

        // contraction
        let mut next_edges = Vec::with_capacity(edges.len());
        for &Edge { id, from, to, cost } in edges.iter() {
            if ids[from] != ids[to] {
                next_edges.push(Edge { id, from: ids[from], to: ids[to], cost: cost - in_cost[to] });
            }
        }

        match self.minimum_cost(scc_cnt, &next_edges, ids[root]) {
            Some((c, e)) => Some((c + total_cost, e)),
            None => None,
        }
    }
}
