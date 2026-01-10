use crate::{
    algorithms::maximum_flow::{edge::MaximumFlowEdge, solvers::dinic::Dinic, solvers::solver::MaximumFlowSolver},
    core::numeric::FlowNum,
    direction::{Directed, Undirected},
    graph::{graph::Graph, ids::NodeId},
};
use crate::prelude::maximum_flow::MaximumFlowError;

pub struct GomoryHu<F> {
    solver: Dinic<F>,
    n: usize,
}

impl<F> GomoryHu<F>
where
    F: FlowNum,
{
    pub fn new<N>(graph: &Graph<Undirected, N, MaximumFlowEdge<F>>) -> Self {
        let dg = to_bidirected(graph);
        let solver = Dinic::<F>::new(&dg);
        Self { solver, n: graph.num_nodes() }
    }

    /// 戻り値: (i, parent[i], weight[i]) の列（i=1..n-1）
    pub fn build(&mut self) -> Result<Vec<(NodeId, NodeId, F)>, MaximumFlowError> {
        let n = self.n;
        if n == 0 {
            return Ok(vec![]);
        }

        let root = NodeId(0);
        let mut parent = vec![root; n];
        let mut weight = vec![F::zero(); n];

        for s_idx in 1..n {
            let s = NodeId(s_idx);
            let t = parent[s_idx];

            // 1) s-t mincut (= maxflow)
            let w = self.solver.solve(s, t)?;
            let source_side = self.solver.minimum_cut()?;

            // 2) parent 更新
            for v_idx in (s_idx + 1)..n {
                if parent[v_idx] == t && source_side[v_idx] {
                    parent[v_idx] = s;
                }
            }

            // 3) swap 規則
            let t_idx = t.index();
            if t != root && source_side[parent[t_idx].index()] {
                let pt = parent[t_idx];
                parent[s_idx] = pt;
                parent[t_idx] = s;

                let tmp = weight[t_idx];
                weight[t_idx] = w;
                weight[s_idx] = tmp;
            } else {
                weight[s_idx] = w;
            }
        }

        let mut tree = Vec::with_capacity(n.saturating_sub(1));
        for i in 1..n {
            tree.push((NodeId(i), parent[i], weight[i]));
        }
        Ok(tree)
    }
}

fn to_bidirected<F: FlowNum, N>(
    g: &Graph<Undirected, N, MaximumFlowEdge<F>>,
) -> Graph<Directed, (), MaximumFlowEdge<F>> {
    let mut dg = Graph::<Directed, (), MaximumFlowEdge<F>>::new_directed();
    dg.add_nodes_with(std::iter::repeat(()).take(g.num_nodes()));

    for e in g.edges() {
        let cap = e.data.upper;
        // u -> v
        dg.add_edge(e.u, e.v, MaximumFlowEdge { upper: cap });
        // v -> u
        dg.add_edge(e.v, e.u, MaximumFlowEdge { upper: cap });
    }
    dg
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::direction::Undirected;
    use crate::graph::{graph::Graph, ids::NodeId};
    use crate::algorithms::maximum_flow::edge::MaximumFlowEdge;

    fn build_tree_adj(n: usize, tree: &[(NodeId, NodeId, i64)]) -> Vec<Vec<(usize, i64)>> {
        let mut adj = vec![Vec::<(usize, i64)>::new(); n];
        for &(u, v, w) in tree {
            let ui = u.index();
            let vi = v.index();
            adj[ui].push((vi, w));
            adj[vi].push((ui, w));
        }
        adj
    }

    fn tree_path_min(adj: &[Vec<(usize, i64)>], s: usize, t: usize) -> i64 {
        if s == t {
            return i64::MAX; // 同一点は比較対象外のため（呼び出し側で避ける想定）
        }
        let n = adj.len();
        let mut parent = vec![usize::MAX; n];
        let mut parent_w = vec![0_i64; n];

        let mut q = std::collections::VecDeque::new();
        parent[s] = s;
        q.push_back(s);

        while let Some(u) = q.pop_front() {
            if u == t {
                break;
            }
            for &(v, w) in &adj[u] {
                if parent[v] == usize::MAX {
                    parent[v] = u;
                    parent_w[v] = w;
                    q.push_back(v);
                }
            }
        }

        assert!(parent[t] != usize::MAX, "tree is disconnected (should never happen)");

        let mut cur = t;
        let mut ans = i64::MAX;
        while cur != s {
            ans = ans.min(parent_w[cur]);
            cur = parent[cur];
        }
        ans
    }

    /// 無向グラフの s-t 最小カットを「全カット列挙」で厳密に計算する（nが小さい前提）
    fn brute_st_mincut_undirected(g: &Graph<Undirected, (), MaximumFlowEdge<i64>>, s: usize, t: usize) -> i64 {
        let n = g.num_nodes();
        assert!(n <= 20, "この brute force は小グラフ用（指数時間）");
        assert!(s < n && t < n && s != t);

        let mut best = i64::MAX;

        // mask は「S 側に入る頂点集合」
        // 条件: s ∈ S, t ∉ S
        for mask in 0_u64..(1_u64 << n) {
            if ((mask >> s) & 1) == 0 {
                continue;
            }
            if ((mask >> t) & 1) == 1 {
                continue;
            }

            let mut cut = 0_i64;
            for e in g.edges() {
                let u = e.u.index();
                let v = e.v.index();
                let in_s_u = ((mask >> u) & 1) == 1;
                let in_s_v = ((mask >> v) & 1) == 1;
                if in_s_u ^ in_s_v {
                    cut += e.data.upper;
                }
            }
            best = best.min(cut);
        }

        best
    }

    fn make_graph(n: usize, edges: &[(usize, usize, i64)]) -> Graph<Undirected, (), MaximumFlowEdge<i64>> {
        let mut g = Graph::<Undirected, (), MaximumFlowEdge<i64>>::new_undirected();
        g.add_nodes_with(std::iter::repeat(()).take(n));
        for &(u, v, cap) in edges {
            g.add_edge(NodeId(u), NodeId(v), MaximumFlowEdge { upper: cap })
                .expect("add_edge failed");
        }
        g
    }

    #[test]
    fn gomory_hu_small_fixed_graph_matches_bruteforce() {
        // 小さな例（手作り）
        // 0--1(3), 1--2(2), 2--3(4), 3--4(1), 0--4(5), 1--3(2)
        let g = make_graph(5, &[(0, 1, 3), (1, 2, 2), (2, 3, 4), (3, 4, 1), (0, 4, 5), (1, 3, 2)]);

        let mut gh = GomoryHu::<i64>::new(&g);
        let tree = gh.build().expect("gomory-hu build failed");

        assert_eq!(tree.len(), g.num_nodes().saturating_sub(1));

        let adj = build_tree_adj(g.num_nodes(), &tree);

        for s in 0..g.num_nodes() {
            for t in (s + 1)..g.num_nodes() {
                let expected = brute_st_mincut_undirected(&g, s, t);
                let got = tree_path_min(&adj, s, t);
                assert_eq!(got, expected, "mismatch for pair (s,t)=({},{})  expected={} got={}", s, t, expected, got);
            }
        }
    }

    // 依存を増やさないために簡易LCGで疑似乱数
    fn lcg_next(x: &mut u64) -> u64 {
        *x = x.wrapping_mul(6364136223846793005).wrapping_add(1);
        *x
    }

    #[test]
    fn gomory_hu_random_small_graphs_match_bruteforce() {
        let n = 7; // 2^7=128 なので全カット列挙が軽い
        let trials = 20;
        let mut seed = 123456789_u64;

        for _ in 0..trials {
            // ランダムに辺を張る（連結性は必須ではないが、木の性質確認のためなるべく繋がるようにする）
            let mut edges = Vec::new();

            // まず鎖で連結に
            for i in 0..(n - 1) {
                let cap = (lcg_next(&mut seed) % 9 + 1) as i64;
                edges.push((i, i + 1, cap));
            }

            // 追加のランダム辺
            for _ in 0..(n * 2) {
                let a = (lcg_next(&mut seed) % (n as u64)) as usize;
                let b = (lcg_next(&mut seed) % (n as u64)) as usize;
                if a == b {
                    continue;
                }
                let (u, v) = if a < b { (a, b) } else { (b, a) };
                let cap = (lcg_next(&mut seed) % 9 + 1) as i64;
                edges.push((u, v, cap));
            }

            let g = make_graph(n, &edges);

            let mut gh = GomoryHu::<i64>::new(&g);
            let tree = gh.build().expect("gomory-hu build failed");
            assert_eq!(tree.len(), n - 1);

            let adj = build_tree_adj(n, &tree);

            for s in 0..n {
                for t in (s + 1)..n {
                    let expected = brute_st_mincut_undirected(&g, s, t);
                    let got = tree_path_min(&adj, s, t);
                    assert_eq!(
                        got, expected,
                        "random test mismatch (s,t)=({},{}) expected={} got={}",
                        s, t, expected, got
                    );
                }
            }
        }
    }
}
