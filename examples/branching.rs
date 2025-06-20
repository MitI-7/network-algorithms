use network_algorithms::algorithms::branching::edmonds::Edmonds;
use network_algorithms::data_structures::UnionFind;
use network_algorithms::edge::weight::WeightEdge;
use network_algorithms::prelude::*;
use std::fs::read_to_string;
use std::path::Path;

fn test() {

    let mut g: Graph<Directed, (), WeightEdge<i64>> = Graph::new_directed();
    let nodes = g.add_nodes(6);
    g.add_directed_edge(nodes[0], nodes[1], 2);
    g.add_directed_edge(nodes[1], nodes[2], 6);
    g.add_directed_edge(nodes[1], nodes[3], 2);
    g.add_directed_edge(nodes[2], nodes[0], 3);
    g.add_directed_edge(nodes[3], nodes[5], 4);
    g.add_directed_edge(nodes[4], nodes[3], 7);
    g.add_directed_edge(nodes[4], nodes[2], 1);
    g.add_directed_edge(nodes[5], nodes[4], 8);

    let mut solver = Edmonds::default();
    let (cost, arborescence) = solver.solve(&g);
    println!("cost:{}", cost);
    for e in arborescence {
        println!("{:?}", g.get_edge(e));
    }
    // assert_eq!(cost, 21);
}

fn main() {
    test()
}
