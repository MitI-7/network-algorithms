use std::fs::read_to_string;
use network_algorithms::algorithms::branching::edmonds::Edmonds;
use network_algorithms::prelude::*;
use network_algorithms::edge::weight::WeightEdge;
use std::path::Path;
use network_algorithms::data_structures::UnionFind;

fn test_solve() {
    let mut graph: Graph<Directed, (), WeightEdge<i128>> = Graph::new_directed();


    let (mut num_nodes, mut num_edges) = (0, 0);
    let mut nodes = Vec::new();
    let mut expected = 0_i128;

    let f = Path::new("tests/branching/AOJ_GRL_2_B/out4.txt");
    read_to_string(&f).unwrap().split('\n').enumerate().for_each(|(i, line)| {
        let line: Vec<&str> = line.split_whitespace().collect();
        if i == 0 {
            (num_nodes, num_edges, expected) = (line[0].parse().unwrap(), line[1].parse().unwrap(), line[2].parse().unwrap());
            nodes = graph.add_nodes(num_nodes);
        } else {
            let (from, to, weight) = (line[0].parse::<usize>().unwrap(), line[1].parse::<usize>().unwrap(), line[2].parse().unwrap());
            graph.add_directed_edge(nodes[from], nodes[to], weight);
        }
    });

    let (cost, branch) = Edmonds::default().solve(&graph);
    println!("{cost}");
    assert_eq!(cost, expected);

    let mut uf = UnionFind::new(graph.num_nodes());
    let mut total = 0;
    let mut used = vec![false; nodes.len()];
    for edge_id in branch {
        let edge = graph.get_edge(edge_id);
        assert!(uf.unite(edge.u.index(), edge.v.index()));  // no cycle
        assert!(!used[edge.v.index()]);
        used[edge.v.index()] = true;
        total += edge.data.weight;
    }
    assert_eq!(total, expected);
}

fn main() {
    test_solve();
}
