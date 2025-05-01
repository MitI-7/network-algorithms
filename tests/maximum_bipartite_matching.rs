use network_algorithms::maximum_bipartite_matching::hopcroft_karp::HopcroftKarp;
use rstest::rstest;
use std::fs::read_to_string;
use std::path::PathBuf;

#[rstest]
fn bipartite_matching(#[files("tests/maximum_bipartite_matching/*/*.txt")] input_file_path: PathBuf) {
    let (mut num_left_nodes, mut num_right_nodes, mut num_edges, mut expected) = (0, 0, 0, 0);

    let mut hk = HopcroftKarp::new(0, 0);

    read_to_string(&input_file_path).unwrap().split('\n').enumerate().for_each(|(i, line)| {
        let line: Vec<&str> = line.split_whitespace().collect();
        if i == 0 {
            (num_left_nodes, num_right_nodes, num_edges, expected) =
                (line[0].parse().unwrap(), line[1].parse().unwrap(), line[2].parse().unwrap(), line[3].parse().unwrap());
            hk = HopcroftKarp::new(num_left_nodes, num_right_nodes);
        } else {
            let (left, right) = (line[0].parse().unwrap(), line[1].parse().unwrap());
            hk.add_edge(left, right);
        }
    });

    let ans = hk.solve();
    assert_eq!(ans.len(), expected);
}
