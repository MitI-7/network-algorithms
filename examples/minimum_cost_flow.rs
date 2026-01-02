use network_algorithms::minimum_cost_flow::prelude::*;

fn primal_network_simplex() {
    let mut graph = MinimumCostFlowGraph::<i32>::new();
    let nodes = graph.add_nodes(5);
    let edges = vec![
        graph.add_edge(nodes[0], nodes[1], 0, 15, 4),
        graph.add_edge(nodes[0], nodes[2], 0, 8, 4),
        graph.add_edge(nodes[1], nodes[2], 0, 20, 2),
        graph.add_edge(nodes[1], nodes[3], 0, 4, 2),
        graph.add_edge(nodes[1], nodes[4], 0, 10, 6),
        graph.add_edge(nodes[2], nodes[3], 0, 15, 1),
        graph.add_edge(nodes[2], nodes[4], 0, 4, 3),
        graph.add_edge(nodes[3], nodes[4], 0, 20, 2),
        graph.add_edge(nodes[4], nodes[2], 0, 5, 3),
    ];
    graph.get_node_mut(nodes[0]).data.b = 20;
    graph.get_node_mut(nodes[3]).data.b = -5;
    graph.get_node_mut(nodes[4]).data.b = -15;

    match SuccessiveShortestPath::default().solve(&mut graph) {
        Ok(result) => {
            println!("minimum cost:{}", result.objective_value);
            for edge_id in edges {
                println!(
                    "{:?}: {}",
                    graph.get_edge(edge_id),
                    result.flows[edge_id.index()]
                );
            }
            assert_eq!(result.objective_value, 150);
        }
        _ => unreachable!(),
    }
}

fn main() {
    primal_network_simplex();
}
