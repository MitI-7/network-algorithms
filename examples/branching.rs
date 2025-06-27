use network_algorithms::branching::{Edmonds, Tarjan};
use network_algorithms::data_structures::UnionFind;
use network_algorithms::edge::weight::WeightEdge;
use network_algorithms::prelude::*;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng}; // ← モジュール名は適宜

/// 頂点数 `n` (1..=10) と最大重み `wmax` (0..=20) を受け取り、
/// 重みを [-wmax, wmax] から一様ランダムに選ぶ完全自由グラフを返す。
pub fn random_weighted_graph(rng: &mut impl Rng, n: usize, wmax: i32) -> Graph<Directed, (), WeightEdge<i32>> {
    assert!((1..=10).contains(&n));
    assert!((0..=20).contains(&wmax));

    // ノードだけ先に確保
    let mut g: Graph<Directed, (), WeightEdge<i32>> = Graph::default();
    let nodes = g.add_nodes(n);

    // 各有向ペア (u, v) について確率 0.3 で辺を張る
    for u in 0..n {
        for v in 0..n {
            if u == v || !rng.gen_bool(0.2) {
                continue;
            }
            let w = rng.gen_range(1..=wmax);
            g.add_edge(nodes[u], nodes[v], WeightEdge { weight: w });
        }
    }
    g
}


fn tarjan_matches_edmonds_on_random_graphs() {
    let mut rng = SmallRng::from_os_rng();

    // 1000 回ランダムに生成して突き合わせ
    for _ in 0..1000000 {
        // let n = rng.gen_range(1..=10);
        let n = 5;
        let g = random_weighted_graph(&mut rng, n, 10);
        if g.num_edges() >= 6 {
            continue;
        }

        let (c_tarjan, _) = Tarjan::<i32>::default().solve(&g);
        let (c_edmonds, _) = Edmonds::<i32>::default().solve(&g);
        println!("{c_tarjan},  {c_edmonds}");
        if c_tarjan != c_edmonds {
            println!("error");
            for e in g.edges {
                println!("{:?}", e);
            }
        }
        
        assert_eq!(c_tarjan, c_edmonds, "cost mismatch on graph with {} nodes", n);
    }
}

fn test() {
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

fn main() {
    // test()
    tarjan_matches_edmonds_on_random_graphs();
}
