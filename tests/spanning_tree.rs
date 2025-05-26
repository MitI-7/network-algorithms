use network_algorithms::algorithms::spanning_tree::edmonds::Edmonds;
use network_algorithms::algorithms::spanning_tree::tarjan::{msa, Edge};
use network_algorithms::edge::weight::WeightEdge;
use network_algorithms::prelude::*;
use rstest::rstest;
use std::fs::read_to_string;
use std::path::PathBuf;
// use std::time::Instant;

#[rstest]
fn directed_spanning_tree(#[files("tests/spanning_tree/*/*.txt")] f: PathBuf) {
    let mut graph = Graph::<Directed, (), WeightEdge<i64>>::new();

    let (mut num_nodes, mut num_edges, mut r) = (0, 0, NodeId(0));
    let mut nodes = Vec::new();
    let mut expected = 0;
    let mut edges = Vec::new();

    read_to_string(&f).unwrap().split('\n').enumerate().for_each(|(i, line)| {
        let line: Vec<&str> = line.split_whitespace().collect();
        if i == 0 {
            (num_nodes, num_edges, r, expected) = (line[0].parse().unwrap(), line[1].parse().unwrap(), NodeId(line[2].parse().unwrap()), line[3].parse().unwrap());
            nodes = graph.add_nodes(num_nodes);
        } else {
            let (from, to, weight) = (line[0].parse::<usize>().unwrap(), line[1].parse::<usize>().unwrap(), line[2].parse().unwrap());
            graph.add_directed_edge(nodes[from], nodes[to], weight);
            edges.push(Edge{from, to, cost: weight});
        }
    });

    
    // edges.push(Edge { from: 0, to: 1, cost: 10 });
    // let start = Instant::now(); // ← 開始時間を記録
    let cost = msa(num_nodes, r.index(), &edges);
    assert_eq!(cost, expected);
    
    
    // match Edmonds::default().solve(&graph, r.index()) {
    //     Some((cost, edges)) => {
    //         let mut total = 0;
    //         let mut used = vec![false; nodes.len()];
    //         for edge_id in edges {
    //             let edge = graph.get_edge(edge_id);
    //             assert!(!used[edge.v.index()]);
    //             used[edge.v.index()] = true;
    //             total += edge.data.weight;
    //         }
    //         assert_eq!(total, expected);
    //         assert_eq!(cost, expected);
    //     }
    //     None => {
    //         panic!("Failed to solve");
    //     }
    // };
    // let elapsed = start.elapsed(); // ← 経過時間
    // println!("[TIME] {}: {:.3?}", input_file_path.file_name().unwrap().to_string_lossy(), elapsed);
}
