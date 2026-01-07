use network_algorithms::minimum_cost_flow::prelude::*;

fn primal_network_simplex() {
    let mut graph = MinimumCostFlowGraph::<i32>::default();
    let nodes = graph.add_nodes(5);
    let edges = vec![
        graph.add_edge(nodes[0], nodes[1], 0, 15, 4).unwrap(),
        graph.add_edge(nodes[0], nodes[2], 0, 8, 4).unwrap(),
        graph.add_edge(nodes[1], nodes[2], 0, 20, 2).unwrap(),
        graph.add_edge(nodes[1], nodes[3], 0, 4, 2).unwrap(),
        graph.add_edge(nodes[1], nodes[4], 0, 10, 6).unwrap(),
        graph.add_edge(nodes[2], nodes[3], 0, 15, 1).unwrap(),
        graph.add_edge(nodes[2], nodes[4], 0, 4, 3).unwrap(),
        graph.add_edge(nodes[3], nodes[4], 0, 20, 2).unwrap(),
        graph.add_edge(nodes[4], nodes[2], 0, 5, 3).unwrap(),
    ];
    graph.get_node_mut(nodes[0]).unwrap().data.b = 20;
    graph.get_node_mut(nodes[3]).unwrap().data.b = -5;
    graph.get_node_mut(nodes[4]).unwrap().data.b = -15;

    match SuccessiveShortestPath::new(&graph).minimum_cost_flow() {
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
