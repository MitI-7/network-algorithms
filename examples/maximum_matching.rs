use network_algorithms::maximum_matching::{Blossom, MaximumMatchingGraph};

fn main() {
    let mut graph = MaximumMatchingGraph::default();
    let nodes = graph.add_nodes(7);

    graph.add_undirected_edge(nodes[2], nodes[0]);
    graph.add_undirected_edge(nodes[0], nodes[5]);
    graph.add_undirected_edge(nodes[5], nodes[6]);
    graph.add_undirected_edge(nodes[6], nodes[1]);
    graph.add_undirected_edge(nodes[1], nodes[0]);
    graph.add_undirected_edge(nodes[1], nodes[3]);
    graph.add_undirected_edge(nodes[3], nodes[4]);
    graph.add_undirected_edge(nodes[1], nodes[4]);

    let matching = Blossom::default().solve(&graph);
    println!("{}", matching.len());
    for edge_id in matching {
        let edge = graph.get_edge(edge_id);
        println!("{} {}", edge.u.index(), edge.v.index());
    }
}
