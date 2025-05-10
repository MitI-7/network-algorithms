use network_algorithms::maximum_bipartite_matching::{HopcroftKarp, MaximumBipartiteMatchingGraph};

fn main() {
    let mut graph = MaximumBipartiteMatchingGraph::default();
    let left_nodes = graph.add_left_nodes(4);
    let right_nodes = graph.add_right_nodes(4);

    graph.add_undirected_edge(left_nodes[1], right_nodes[1]);
    graph.add_undirected_edge(left_nodes[2], right_nodes[2]);
    graph.add_undirected_edge(left_nodes[0], right_nodes[0]);
    graph.add_undirected_edge(left_nodes[3], right_nodes[1]);
    graph.add_undirected_edge(left_nodes[1], right_nodes[2]);
    graph.add_undirected_edge(left_nodes[2], right_nodes[0]);
    graph.add_undirected_edge(left_nodes[3], right_nodes[2]);

    let matching = HopcroftKarp::default().solve(&graph);

    println!("{}", matching.len());
    for &edge_id in matching.iter() {
        let edge = &graph.get_edge(edge_id).unwrap();
        println!("{} {}", edge.u.index(), edge.v.index());
    }
}
