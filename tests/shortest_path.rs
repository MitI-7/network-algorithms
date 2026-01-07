use network_algorithms::algorithms::shortest_path::prelude::*;
use rstest::rstest;
use std::fs::read_to_string;
use std::path::PathBuf;

enum Solver {
    Dijkstra,
    BellmanFord,
}

#[rstest]
#[case::dijkstra(Solver::Dijkstra)]
#[case::bellman_ford(Solver::BellmanFord)]
fn shortest_path(#[files("tests/shortest_path/*/*.txt")] input_file_path: PathBuf, #[case] solver: Solver) {
    let mut graph = ShortestPathGraph::<i32>::default();
    let (mut num_nodes, mut num_edges, mut s) = (0, 0, 0);
    let mut nodes = Vec::new();
    let mut expected: Vec<Option<usize>> = Vec::new();

    read_to_string(&input_file_path)
        .unwrap()
        .split('\n')
        .enumerate()
        .for_each(|(i, line)| {
            let line: Vec<&str> = line.split_whitespace().collect();
            if i == 0 {
                (num_nodes, num_edges, s) =
                    (line[0].parse().unwrap(), line[1].parse().unwrap(), line[2].parse::<usize>().unwrap());
                nodes = graph.add_nodes(num_nodes);
            } else if i == 1 {
                for j in 0..num_nodes {
                    assert_eq!(line.len(), num_nodes);
                    expected.push(line[j].parse().ok());
                }
            } else {
                let (from, to, weight) = (
                    line[0].parse::<usize>().unwrap(),
                    line[1].parse::<usize>().unwrap(),
                    line[2].parse::<i32>().unwrap(),
                );
                graph.add_edge(nodes[from], nodes[to], weight);
            }
        });

    let _actual = match solver {
        Solver::Dijkstra => Dijkstra::new(&graph).solve(nodes[s]),
        Solver::BellmanFord => BellmanFord::new(&graph).solve(nodes[s]),
    };

    // assert_eq!(actual.unwrap(), expected);
}
