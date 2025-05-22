use network_algorithms::algorithms::spanning_tree::edmonds::Edmonds;
use network_algorithms::edge::weight::WeightEdge;
use network_algorithms::prelude::*;
use rstest::rstest;
use std::fs::read_to_string;
use std::path::PathBuf;
use std::time::Instant;

#[rstest]
fn directed_spanning_tree(#[files("tests/spanning_tree/*/*.txt")] input_file_path: PathBuf) {
    let mut graph = Graph::<Directed, (), WeightEdge<u128>>::new();

    let (mut num_nodes, mut num_edges, mut r) = (0, 0, NodeId(0));
    let mut nodes = Vec::new();
    let mut expected = 0;

    read_to_string(&input_file_path).unwrap().split('\n').enumerate().for_each(|(i, line)| {
        let line: Vec<&str> = line.split_whitespace().collect();
        if i == 0 {
            (num_nodes, num_edges, r, expected) = (line[0].parse().unwrap(), line[1].parse().unwrap(), NodeId(line[2].parse().unwrap()), line[3].parse().unwrap());
            nodes = graph.add_nodes(num_nodes);
        } else {
            let (from, to, weight) = (line[0].parse::<usize>().unwrap(), line[1].parse::<usize>().unwrap(), line[2].parse().unwrap());
            graph.add_directed_edge(nodes[from], nodes[to], weight);
        }
    });

    let start = Instant::now(); // ← 開始時間を記録
    match Edmonds::default().solve(&graph, r.index()) {
        Some((cost, edges)) => {
            assert_eq!(cost, expected);
            let mut t = 0;
            for edge_id in edges {
                let edge = graph.get_edge(edge_id);
                t += edge.data.weight;
            }
            assert_eq!(t, expected);
        }
        None => {
            panic!("Failed to solve");
        }
    };
    let elapsed = start.elapsed(); // ← 経過時間
    println!("[TIME] {}: {:.3?}", input_file_path.file_name().unwrap().to_string_lossy(), elapsed);
}
