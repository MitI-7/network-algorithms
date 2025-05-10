use network_algorithms::minimum_cost_flow::{MinimumCostFlowGraph, BestEligibleArcPivotRule, PrimalNetworkSimplex};

fn primal_network_simplex() {
    let mut graph = MinimumCostFlowGraph::<i32>::default();
    let nodes = graph.add_nodes(4);

    let edges = vec![
        graph.add_directed_edge(nodes[0], nodes[1], 0, 2, 1),
        graph.add_directed_edge(nodes[0], nodes[2], 0, 1, 2),
        graph.add_directed_edge(nodes[1], nodes[2], 0, 1, 1),
        graph.add_directed_edge(nodes[1], nodes[3], 0, 1, 3),
        graph.add_directed_edge(nodes[2], nodes[3], 0, 2, 1),
    ];

    graph.nodes[0].b = 2;
    graph.nodes[3].b = -2;

    let pivot = BestEligibleArcPivotRule::default();
    match PrimalNetworkSimplex::<i32>::default().set_pivot(pivot).solve(&mut graph) {
        Ok(value) => {
            println!("minimum cost:{}", value);
            for edge_id in edges {
                println!("{:?}", graph.get_edge(edge_id));
            }
        }
        _ => unreachable!(),
    }
}

fn main() {
    primal_network_simplex();
}
