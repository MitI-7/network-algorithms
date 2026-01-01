use network_algorithms::graph::{
    direction::Directed,
    graph::Graph,
};

fn main() {
    let mut graph: Graph<Directed, (), ()> = Graph::new_directed();
    let nodes = graph.add_nodes(5);
    graph.add_edge(nodes[0], nodes[1], ());
}
