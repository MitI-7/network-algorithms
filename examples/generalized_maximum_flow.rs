use network_algorithms::graph::generalized_maximum_flow_graph::Graph;
use network_algorithms::generalized_maximum_flow::primal_dual_push_relabel::PrimalDualPushRelabel;

fn main() {
    let epsilon = 0.01;
    let mut graph = Graph::default();
    graph.add_nodes(8);

    graph.add_directed_edge(0, 1, 12.0, 0.7);
    graph.add_directed_edge(0, 2, 3.0, 0.9);
    graph.add_directed_edge(0, 3, 4.0, 0.8);

    graph.add_directed_edge(1, 4, 3.0, 0.5);
    graph.add_directed_edge(1, 5, 5.0, 0.8);

    graph.add_directed_edge(2, 1, 2.7, 1.0);
    graph.add_directed_edge(2, 3, 20.0 / 9.0, 0.9);
    graph.add_directed_edge(2, 5, 5.0, 0.7);

    graph.add_directed_edge(3, 5, 1.0, 1.0);
    graph.add_directed_edge(3, 6, 2.0, 0.7);

    graph.add_directed_edge(4, 7, 2.0, 0.5);

    graph.add_directed_edge(5, 4, 1.0, 0.5);
    graph.add_directed_edge(5, 6, 6.0, 0.7);
    graph.add_directed_edge(5, 7, 1.3, 1.0);

    graph.add_directed_edge(6, 7, 7.0, 1.0);

    PrimalDualPushRelabel::new(epsilon).solve(0, 7, &mut graph);

    let actual = graph.maximum_flow(7);

    let expected = 7.363;
    assert!(expected * (1.0 - epsilon) <= actual && actual <= expected, "{}/{}", actual, expected);
}
