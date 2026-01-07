use network_algorithms::maximum_matching::prelude::*;
use rstest::rstest;
use std::fs::read_to_string;
use std::path::PathBuf;

#[rstest]
fn maximum_matching(#[files("tests/maximum_matching/*/*.txt")] input_file_path: PathBuf) {
    let (mut n, mut m, mut expected) = (0, 0, 0);

    let mut graph = MaximumMatchingGraph::default();
    let mut nodes = Vec::new();
    read_to_string(&input_file_path)
        .unwrap()
        .split('\n')
        .enumerate()
        .for_each(|(i, line)| {
            let line: Vec<&str> = line.split_whitespace().collect();
            if i == 0 {
                (n, m, expected) = (line[0].parse().unwrap(), line[1].parse().unwrap(), line[2].parse().unwrap());
                nodes = graph.add_nodes(n);
            } else {
                let (u, v) = (line[0].parse::<usize>().unwrap(), line[1].parse::<usize>().unwrap());
                graph.add_edge(nodes[u], nodes[v], ());
            }
        });

    let matching = Blossom::default().solve(&graph);
    assert_eq!(matching.len(), expected);

    let mut used = vec![false; graph.num_nodes()].into_boxed_slice();
    for edge_id in matching {
        let e = graph.get_edge(edge_id).unwrap();
        assert!(!used[e.u.index()] && !used[e.v.index()]);
        used[e.u.index()] = true;
        used[e.v.index()] = true;
    }
}
