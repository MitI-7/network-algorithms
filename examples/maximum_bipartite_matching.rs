// use network_algorithms::maximum_bipartite_matching::{BipartiteGraph, HopcroftKarp};
// 
// use std::io::*;
// use std::str::FromStr;
// 
// fn read<T: FromStr>() -> T {
//     let stdin = stdin();
//     let stdin = stdin.lock();
//     let token: String = stdin
//         .bytes()
//         .map(|c| c.expect("failed to read char") as char)
//         .skip_while(|c| c.is_whitespace())
//         .take_while(|c| !c.is_whitespace())
//         .collect();
//     token.parse().ok().expect("failed to parse token")
// }
// 
// // https://judge.yosupo.jp/problem/bipartitematching
// fn main() {
//     let (num_left_nodes, num_right_nodes, num_edges): (usize, usize, usize) = (read(), read(), read());
// 
//     let mut graph = BipartiteGraph::default();
//     let left_nodes = graph.add_left_nodes(num_left_nodes);
//     let right_nodes = graph.add_right_nodes(num_right_nodes);
//     for _ in 0..num_edges {
//         let (a, b): (usize, usize) = (read(), read());
//         graph.add_undirected_edge(left_nodes[a], right_nodes[b]);
//     }
//     let matching = HopcroftKarp::default().solve(&graph);
// 
//     println!("{}", matching.len());
//     for &edge_id in matching.iter() {
//         let edge = &graph.get_edge(edge_id).unwrap();
//         println!("{} {}", edge.u, edge.v);
//     }
// }

fn main() {
    
}