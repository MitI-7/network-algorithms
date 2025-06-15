use crate::core::direction::Directed;
use crate::core::graph::Graph;
use crate::edge::weight::WeightEdge;
use crate::prelude::EdgeId;
use crate::traits::{IntNum, Zero};
use std::marker::PhantomData;
use std::ops::Neg;

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
    num_edges: usize,
    phantom_data: PhantomData<W>,
}

impl<W> Edmonds<W>
where
    W: IntNum + Zero + Neg<Output = W> + Copy,
{
    pub fn solve(&mut self, graph: &Graph<Directed, (), WeightEdge<W>>) -> (W, Vec<EdgeId>) {
        self.num_edges = graph.num_edges();
        let mut edges = Vec::with_capacity(graph.num_edges());
        for (i, edge) in graph.edges.iter().enumerate() {
            if edge.data.weight <= W::zero() {
                continue;
            }
            edges.push(Edge { id: EdgeId(i), from: edge.u.index(), to: edge.v.index(), cost: edge.data.weight });
        }

        self.maximum_branching(graph.num_nodes(), &edges)
    }

    fn maximum_branching(&self, num_nodes: usize, edges: &Vec<Edge<W>>) -> (W, Vec<EdgeId>) {
        let mini = -W::max_value();

        let mut in_cost = vec![mini; num_nodes];
        let mut parent = vec![usize::MAX; num_nodes];
        let mut in_edge_id = vec![None; num_nodes];
        let mut edge_id_to_node = vec![None; self.num_edges];
        for &Edge { id, from, to, cost } in edges.iter() {
            if from == to || cost <= W::zero() {
                continue;
            }

            if from != to && cost > in_cost[to] {
                in_cost[to] = cost;
                parent[to] = from;
                in_edge_id[to] = Some(id);
            }
            edge_id_to_node[id.index()] = Some(to);
        }

        let mut is_critical = vec![false; self.num_edges];
        for s in in_edge_id.iter() {
            if s.is_some() {
                is_critical[s.unwrap().index()] = true;
            }
        }

        // decomposition of strongly connected components
        let mut ids = vec![usize::MAX; num_nodes];
        let mut scc_cnt = 0;
        {
            let mut stamp = vec![usize::MAX; num_nodes];
            for u in 0..num_nodes {
                let mut v = u;

                while v != usize::MAX && stamp[v] != u && ids[v] == usize::MAX {
                    stamp[v] = u;
                    v = parent[v];
                }
                // find cycle
                if v != usize::MAX && ids[v] == usize::MAX {
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
                // ① 根でない頂点 (= parent[u] が存在する頂点) の in_cost をすべて加算
                let mut cost = W::zero();
                for (u, &p) in parent.iter().enumerate() {
                    if p != usize::MAX {
                        cost += in_cost[u];
                    }
                }

                return (cost, in_edge_id.iter().filter_map(|&edge| edge).collect());
            }

            for u in 0..num_nodes {
                if ids[u] == usize::MAX {
                    ids[u] = scc_cnt;
                    scc_cnt += 1;
                }
            }
        }

        let mut num_components = vec![0; scc_cnt];
        for u in 0..num_nodes {
            num_components[ids[u]] += 1;
        }

        // サイクル最小の重み
        let mut mini_cost_in_cycle = vec![W::max_value(); scc_cnt];
        let mut mini_cost_id_in_cycle = vec![usize::MAX; scc_cnt];
        for &Edge { id, from, to, cost } in edges.iter() {
            if is_critical[id.index()] {
                // サイクル
                if ids[from] == ids[to] {
                    let cycle_no = ids[to];
                    if cost < mini_cost_in_cycle[cycle_no] {
                        mini_cost_in_cycle[cycle_no] = cost;
                        mini_cost_id_in_cycle[cycle_no] = id.index();
                    }
                }
            }
        }

        let mut total_cost = W::zero();
        for u in 0..num_nodes {
            // サイクル（サイズ > 1）の頂点だけを対象
            if num_components[ids[u]] > 1 && parent[u] != usize::MAX {
                total_cost += in_cost[u];
            }
        }

        // ③ サイクルごとに最小 in_cost を 1 本だけ引く（ここは現行のままで OK）
        for (cid, &min_in_cycle) in mini_cost_in_cycle.iter().enumerate() {
            if num_components[cid] > 1 && min_in_cycle != W::max_value() {
                total_cost -= min_in_cycle;
            }
        }

        // contraction
        let mut next_edges = Vec::with_capacity(edges.len());
        for &Edge { id, from, to, cost } in edges.iter() {
            // edge is in cycle
            if ids[from] == ids[to] {
                continue;
            }

            // to cycle
            if num_components[ids[to]] > 1 {
                next_edges.push(Edge { id, from: ids[from], to: ids[to], cost: cost - in_cost[to] + mini_cost_in_cycle[ids[to]] });
            } else {
                next_edges.push(Edge { id, from: ids[from], to: ids[to], cost });
            }
        }

        assert!(scc_cnt < num_nodes);
        let (cost, mut arborescence) = self.maximum_branching(scc_cnt, &next_edges);

        let mut node_has_entry_edge = vec![false; num_nodes];
        let mut cycle_has_entry_edge = vec![false; scc_cnt];
        for &edge_id in arborescence.iter() {
            if let Some(to) = edge_id_to_node[edge_id.index()] {
                node_has_entry_edge[to] = true;
                cycle_has_entry_edge[ids[to]] = true;
            }
        }

        for (u, edge_id) in in_edge_id.iter().enumerate() {
            if parent[u] == usize::MAX {
                continue;
            }

            // サイクルを展開
            if ids[u] == ids[parent[u]] {
                // サイクルに入る辺がある場合は，サイクルに入る辺と同じ行き先をもつ辺以外を採用
                if cycle_has_entry_edge[ids[u]] && !node_has_entry_edge[u] {
                    arborescence.push(edge_id.unwrap());
                }

                // サイクルに入る辺がない場合は，最小以外の辺を採用
                if !cycle_has_entry_edge[ids[u]] && edge_id.unwrap().index() != mini_cost_id_in_cycle[ids[u]] {
                    arborescence.push(edge_id.unwrap());
                }
            }
        }
        (cost + total_cost, arborescence)
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_solve() {
        let mut g: Graph<Directed, (), WeightEdge<i64>> = Graph::new_directed();
        let nodes = g.add_nodes(4);
        g.add_directed_edge(nodes[0], nodes[1], 3);
        g.add_directed_edge(nodes[0], nodes[2], 2);
        g.add_directed_edge(nodes[2], nodes[0], 1);
        g.add_directed_edge(nodes[2], nodes[3], 1);
        g.add_directed_edge(nodes[3], nodes[0], 1);
        g.add_directed_edge(nodes[3], nodes[1], 5);

        let (cost, arborescence) = Edmonds::default().solve(&g);
        println!("cost:{}", cost);
        for e in arborescence {
            println!("{:?}", g.get_edge(e));
        }
    }
}
