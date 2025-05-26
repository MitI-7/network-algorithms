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
    num_nodes: usize,
    num_edges: usize,
    phantom_data: PhantomData<W>,
}

impl<W> Edmonds<W>
where
    W: IntNum + Zero,
{
    pub fn solve(&mut self, graph: &Graph<Directed, (), WeightEdge<W>>, root: usize) -> Option<(W, Vec<EdgeId>)> {
        self.num_nodes = graph.num_nodes();
        self.num_edges = graph.num_edges();

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
        let mut in_edge_id = vec![None; num_nodes];
        let mut edge_id_to_node = vec![None; self.num_edges];
        for &Edge { id, from, to, cost } in edges.iter() {
            if from != to && cost < in_cost[to] {
                in_cost[to] = cost;
                parent[to] = from;
                in_edge_id[to] = Some(id);
            }
            edge_id_to_node[id.index()] = Some(to);
        }
        in_cost[root] = W::zero();
        in_edge_id[root] = None;
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
                return Some((total_cost, in_edge_id.iter().filter_map(|&edge| edge).collect()));
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
            Some((cost, mut arborescence)) => {
                let mut has_entry_edge = vec![false; num_nodes];
                for &edge_id in arborescence.iter() {
                    if let Some(to) = edge_id_to_node[edge_id.index()] {
                        has_entry_edge[to] = true;
                    }
                }

                for (u, edge_id) in in_edge_id.iter().enumerate() {
                    if u == root {
                        continue;
                    }

                    // cycle
                    if !has_entry_edge[u] && ids[u] == ids[parent[u]] {
                        arborescence.push(edge_id.unwrap());
                    }
                }
                Some((cost + total_cost, arborescence))
            }
            None => None,
        }
    }
}
