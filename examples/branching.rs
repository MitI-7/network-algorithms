use network_algorithms::branching::{Edmonds, Tarjan};
use network_algorithms::data_structures::UnionFind;
use network_algorithms::edge::weight::WeightEdge;
use network_algorithms::prelude::*;
use std::fs::read_to_string;
use std::path::Path;

fn test() {

    let mut g: Graph<Directed, (), WeightEdge<i32>> = Graph::new_directed();
    let nodes = g.add_nodes(10);
    g.add_directed_edge(nodes[0], nodes[2], 1175);
    g.add_directed_edge(nodes[2], nodes[1], 6460);
    g.add_directed_edge(nodes[1], nodes[4], 4761);
    g.add_directed_edge(nodes[4], nodes[8], 8294);
    g.add_directed_edge(nodes[8], nodes[5], 5954);
    g.add_directed_edge(nodes[5], nodes[9], 4564);
    g.add_directed_edge(nodes[9], nodes[7], 7208);
    g.add_directed_edge(nodes[5], nodes[3], 6028);
    g.add_directed_edge(nodes[1], nodes[0], 3092);
    g.add_directed_edge(nodes[5], nodes[0], 6527);
    g.add_directed_edge(nodes[7], nodes[5], 823);
    g.add_directed_edge(nodes[0], nodes[8], 8252);
    g.add_directed_edge(nodes[6], nodes[7], 1343);
    g.add_directed_edge(nodes[3], nodes[2], 8365);
    g.add_directed_edge(nodes[9], nodes[6], 4996);

    let mut solver = Tarjan::default();
    let (cost, arborescence) = solver.solve(&g);
    let mut used = vec![false; g.num_nodes()];
    for edge_id in arborescence {
        println!("{:?}", g.get_edge(edge_id));
        assert!(!used[g.get_edge(edge_id).v.index()]);
        used[g.get_edge(edge_id).v.index()] = true;
    }
    assert_eq!(cost, 58396);
}

fn main() {
    test()
}
