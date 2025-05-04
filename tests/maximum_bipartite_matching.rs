use network_algorithms::maximum_bipartite_matching::{BipartiteGraph, HopcroftKarp, WarmStart};
use rstest::rstest;
use std::fs::read_to_string;
use std::path::PathBuf;

#[rstest]
fn bipartite_matching(#[files("tests/maximum_bipartite_matching/*/*.txt")] input_file_path: PathBuf) {
    let (mut num_left_nodes, mut num_right_nodes, mut num_edges, mut expected) = (0, 0, 0, 0);

    let mut graph = BipartiteGraph::default();

    read_to_string(&input_file_path).unwrap().split('\n').enumerate().for_each(|(i, line)| {
        let line: Vec<&str> = line.split_whitespace().collect();
        if i == 0 {
            (num_left_nodes, num_right_nodes, num_edges, expected) =
                (line[0].parse().unwrap(), line[1].parse().unwrap(), line[2].parse().unwrap(), line[3].parse().unwrap());
            graph.add_left_nodes(num_left_nodes);
            graph.add_right_nodes(num_right_nodes);
        } else {
            let (left, right) = (line[0].parse().unwrap(), line[1].parse().unwrap());
            graph.add_undirected_edge(left, right);
        }
    });

    let matching = HopcroftKarp::default().set_warm_start(WarmStart::KarpSipser).solve(&graph);
    assert_eq!(matching.len(), expected);

    let (mut used_u, mut used_v) = (vec![false; num_left_nodes].into_boxed_slice(), vec![false; num_right_nodes].into_boxed_slice());
    for edge_id in matching {
        let edge = graph.get_edge(edge_id).unwrap();
        assert!(!used_u[edge.u] && !used_v[edge.v]);
        used_u[edge.u] = true;
        used_v[edge.v] = true;
    }
}
