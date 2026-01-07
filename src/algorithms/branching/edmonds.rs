//! Algorithms for finding optimum branchings
//! Based on the following works:
//! [1] J. Edmonds, “Optimum branchings,” J. RES. NATL. BUR. STAN. SECT. B. MATH. MATH. PHYS., vol. 71B, no. 4, p. 233, Oct. 1967, doi: 10.6028/jres.071B.032.
//! [2] R. M. Karp, “A simple derivation of edmonds’ algorithm for optimum branchings,” Networks, vol. 1, no. 3, pp. 265–272, Jan. 1971, doi: 10.1002/net.3230010305.

use crate::graph::direction::Directed;
use crate::graph::graph::Graph;
use crate::algorithms::branching::edge::WeightEdge;
use crate::graph::ids::EdgeId;
use std::marker::PhantomData;
use crate::core::numeric::FlowNum;

#[derive(Clone, Debug)]
struct Edge<W> {
    id: EdgeId,
    from: usize,
    to: usize,
    cost: W,
}

// O(nm)
#[derive(Default)]
pub struct Edmonds<W> {
    num_edges: usize,
    phantom_data: PhantomData<W>,
}

impl<W> Edmonds<W>
where
    W: FlowNum,
{
    pub fn solve(&mut self, graph: &Graph<Directed, (), WeightEdge<W>>) -> (W, Vec<EdgeId>) {
        self.num_edges = graph.num_edges();
        let mut edges = Vec::with_capacity(graph.num_edges());
        for (id, edge) in graph.edges().enumerate() {
            edges.push(Edge { id: EdgeId(id), from: edge.u.index(), to: edge.v.index(), cost: edge.data.weight });
        }

        self.maximum_branching(graph.num_nodes(), &edges)
    }

    fn maximum_branching(&self, num_nodes: usize, edges: &[Edge<W>]) -> (W, Vec<EdgeId>) {
        let mut critical_edge = vec![(usize::MAX, W::zero(), EdgeId(0)); num_nodes]; // from, cost, id
        let mut edge_id_to_node = vec![None; self.num_edges];
        for &Edge { id, from, to, cost } in edges.iter() {
            if from == to || cost <= W::zero() {
                continue;
            }

            if cost > critical_edge[to].1 {
                critical_edge[to] = (from, cost, id);
            }
            edge_id_to_node[id.index()] = Some(to);
        }

        // decomposition of strongly connected components
        let mut ids = vec![usize::MAX; num_nodes];
        let mut num_scc = 0;
        {
            let mut stamp = vec![usize::MAX; num_nodes];
            for u in 0..num_nodes {
                let mut v = u;

                while v != usize::MAX && stamp[v] != u && ids[v] == usize::MAX {
                    stamp[v] = u;
                    v = critical_edge[v].0;
                }
                // find cycle
                if v != usize::MAX && ids[v] == usize::MAX {
                    let mut w = critical_edge[v].0;
                    ids[v] = num_scc;
                    while w != v {
                        ids[w] = num_scc;
                        w = critical_edge[w].0;
                    }
                    num_scc += 1;
                }
            }

            // no cycle
            if num_scc == 0 {
                let total_cost = critical_edge.iter().fold(W::zero(), |t, (_, c, _)| t + *c);
                let edge_ids: Vec<EdgeId> = critical_edge
                    .iter()
                    .filter_map(|&(from, _, id)| if from != usize::MAX { Some(id) } else { None })
                    .collect();
                return (total_cost, edge_ids);
            }

            for u in 0..num_nodes {
                if ids[u] == usize::MAX {
                    ids[u] = num_scc;
                    num_scc += 1;
                }
            }
        }

        let mut num_components = vec![0; num_scc];
        for u in 0..num_nodes {
            num_components[ids[u]] += 1;
        }

        let mut total_cost = W::zero();
        let mut mini_cost_in_cycle = vec![W::max_value(); num_scc];
        let mut mini_cost_id_in_cycle = vec![usize::MAX; num_scc];
        for (to, &(from, cost, id)) in critical_edge.iter().enumerate() {
            if from != usize::MAX && ids[from] == ids[to] {
                let cycle_no = ids[to];
                if cost < mini_cost_in_cycle[cycle_no] {
                    mini_cost_in_cycle[cycle_no] = cost;
                    mini_cost_id_in_cycle[cycle_no] = id.index();
                }
                total_cost += cost;
            }
        }

        for c in 0..num_scc {
            if mini_cost_in_cycle[c] != W::max_value() {
                total_cost -= mini_cost_in_cycle[c];
            }
        }

        // contraction
        let mut next_edges = Vec::with_capacity(edges.len());
        for &Edge { id, from, to, cost } in edges.iter() {
            // edge in cycle is ignored
            if ids[from] == ids[to] {
                continue;
            }

            // edge is into cycle
            if num_components[ids[to]] > 1 {
                next_edges.push(Edge { id, from: ids[from], to: ids[to], cost: cost - critical_edge[to].1 + mini_cost_in_cycle[ids[to]] });
            } else {
                next_edges.push(Edge { id, from: ids[from], to: ids[to], cost });
            }
        }

        assert!(num_scc < num_nodes);
        let (cost, mut branching) = self.maximum_branching(num_scc, &next_edges);

        let mut node_has_entry_edge = vec![false; num_nodes];
        let mut cycle_has_entry_edge = vec![false; num_scc];
        for &edge_id in branching.iter() {
            if let Some(to) = edge_id_to_node[edge_id.index()] {
                node_has_entry_edge[to] = true;
                cycle_has_entry_edge[ids[to]] = true;
            }
        }

        for (to, &(_from, _cost, edge_id)) in critical_edge.iter().enumerate() {
            if critical_edge[to].0 == usize::MAX {
                continue;
            }

            // expand cycle
            if ids[to] == ids[critical_edge[to].0] {
                // サイクルに入る辺がある場合は，サイクルに入る辺と同じ行き先をもつ辺以外を採用
                if cycle_has_entry_edge[ids[to]] && !node_has_entry_edge[to] {
                    branching.push(edge_id);
                }

                // サイクルに入る辺がない場合は，最小以外の辺を採用
                if !cycle_has_entry_edge[ids[to]] && edge_id.index() != mini_cost_id_in_cycle[ids[to]] {
                    branching.push(edge_id);
                }
            }
        }
        (cost + total_cost, branching)
    }
}
