use network_algorithms::maximum_matching::{Blossom, Graph};
use rstest::rstest;
use std::fs::read_to_string;
use std::path::PathBuf;

#[rstest]
fn maximum_matching(#[files("tests/maximum_matching/*/*.txt")] input_file_path: PathBuf) {
    let (mut n, mut m, mut expected) = (0, 0, 0);

    let mut graph = Graph::default();
    read_to_string(&input_file_path).unwrap().split('\n').enumerate().for_each(|(i, line)| {
        let line: Vec<&str> = line.split_whitespace().collect();
        if i == 0 {
            (n, m, expected) = (line[0].parse().unwrap(), line[1].parse().unwrap(), line[2].parse().unwrap());
            graph.add_nodes(n);
        } else {
            let (u, v) = (line[0].parse().unwrap(), line[1].parse().unwrap());
            graph.add_undirected_edge(u, v);
        }
    });

    let actual = Blossom::default().solve(&graph);
    assert_eq!(actual, expected);
}
