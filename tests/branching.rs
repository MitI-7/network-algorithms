use network_algorithms::algorithms::branching::prelude::*;
use network_algorithms::data_structures::union_find::UnionFind;
use network_algorithms::{Graph, direction::Directed};
use rstest::rstest;
use std::fs::read_to_string;
use std::path::PathBuf;

#[rstest]
fn branching(#[files("tests/branching/*/*.txt")] f: PathBuf) {
    // if f.to_str().unwrap().contains("random_02") {
    //     return;
    // }
    // if f.to_str().unwrap().contains("random_04") {
    //     return;
    // }

    let mut graph = Graph::<Directed, (), WeightEdge<i128>>::default();

    let (mut num_nodes, mut num_edges) = (0, 0);
    let mut nodes = Vec::new();
    let mut expected = 0_i128;

    read_to_string(&f)
        .unwrap()
        .split('\n')
        .enumerate()
        .for_each(|(i, line)| {
            let line: Vec<&str> = line.split_whitespace().collect();
            if i == 0 {
                (num_nodes, num_edges, expected) =
                    (line[0].parse().unwrap(), line[1].parse().unwrap(), line[2].parse().unwrap());
                nodes = graph.add_nodes(num_nodes);
            } else {
                let (from, to, weight) = (
                    line[0].parse::<usize>().unwrap(),
                    line[1].parse::<usize>().unwrap(),
                    line[2].parse::<i128>().unwrap(),
                );
                graph.add_edge(nodes[from], nodes[to], WeightEdge { weight });
            }
        });

    // let (cost, branching) = Edmonds::default().solve(&graph);
    let (cost, branching) = Tarjan::default().solve(&graph);
    let mut total = 0;
    let mut uf = UnionFind::new(graph.num_nodes());
    let mut indegree = vec![0; nodes.len()];
    for edge_id in branching {
        let edge = graph.get_edge(edge_id).unwrap();
        indegree[edge.v.index()] += 1;
        total += edge.data.weight;
        assert!(uf.union(edge.u.index(), edge.v.index())); // no cycle
        assert!(indegree[edge.v.index()] <= 1); // in-degree of each node is at most 1
    }
    assert_eq!(cost, expected);
    assert_eq!(total, expected);
}
