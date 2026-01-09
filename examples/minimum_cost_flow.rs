use network_algorithms::ids::EdgeId;
use network_algorithms::minimum_cost_flow::prelude::*;

fn make_graph() -> (MinimumCostFlowGraph<i32>, Vec<EdgeId>) {
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
    graph.set_excess(nodes[0], 20);
    graph.set_excess(nodes[3], -5);
    graph.set_excess(nodes[4], -15);

    (graph, edges)
}

fn primal_network_simplex() {
    let (graph, edges) = make_graph();
    let mut solver = PrimalNetworkSimplex::new(&graph);

    match solver.solve() {
        Ok(objective_value) => {
            println!("minimum cost:{}", objective_value);
            for edge_id in edges {
                println!("{:?}: {}", graph.get_edge(edge_id), solver.flow(edge_id).unwrap());
            }
            assert_eq!(objective_value, 150);
        }
        _ => unreachable!(),
    }
}

fn main() {
    primal_network_simplex();
}
