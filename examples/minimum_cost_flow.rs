use network_algorithms::algorithms::minimum_cost_flow::{
    edge::MinimumCostFlowEdge, node::MinimumCostFlowNode, solver::MinimumCostFlowSolver,
    successive_shortest_path::SuccessiveShortestPath,
};
use network_algorithms::graph::{direction::Directed, graph::Graph};

fn primal_network_simplex() {
    let mut graph: Graph<Directed, MinimumCostFlowNode<i32>, MinimumCostFlowEdge<i32>> =
        Graph::new_directed();
    let nodes = graph.add_nodes(5);
    let edges = vec![
        graph.add_edge(
            nodes[0],
            nodes[1],
            MinimumCostFlowEdge {
                lower: 0,
                upper: 15,
                cost: 4,
            },
        ),
        graph.add_edge(
            nodes[0],
            nodes[2],
            MinimumCostFlowEdge {
                lower: 0,
                upper: 8,
                cost: 4,
            },
        ),
        graph.add_edge(
            nodes[1],
            nodes[2],
            MinimumCostFlowEdge {
                lower: 0,
                upper: 20,
                cost: 2,
            },
        ),
        graph.add_edge(
            nodes[1],
            nodes[3],
            MinimumCostFlowEdge {
                lower: 0,
                upper: 4,
                cost: 2,
            },
        ),
        graph.add_edge(
            nodes[1],
            nodes[4],
            MinimumCostFlowEdge {
                lower: 0,
                upper: 10,
                cost: 6,
            },
        ),
        graph.add_edge(
            nodes[2],
            nodes[3],
            MinimumCostFlowEdge {
                lower: 0,
                upper: 15,
                cost: 1,
            },
        ),
        graph.add_edge(
            nodes[2],
            nodes[4],
            MinimumCostFlowEdge {
                lower: 0,
                upper: 4,
                cost: 3,
            },
        ),
        graph.add_edge(
            nodes[3],
            nodes[4],
            MinimumCostFlowEdge {
                lower: 0,
                upper: 20,
                cost: 2,
            },
        ),
        graph.add_edge(
            nodes[4],
            nodes[2],
            MinimumCostFlowEdge {
                lower: 0,
                upper: 5,
                cost: 3,
            },
        ),
    ];
    graph.get_node_mut(nodes[0]).data.b = 20;
    graph.get_node_mut(nodes[3]).data.b = -5;
    graph.get_node_mut(nodes[4]).data.b = -15;

    match SuccessiveShortestPath::default().solve(&mut graph) {
        Ok(result) => {
            println!("minimum cost:{}", result.objective_value);
            for edge_id in edges {
                println!("{:?}: {}", graph.get_edge(edge_id), result.flows[edge_id.index()]);
            }
            assert_eq!(result.objective_value, 150);
        }
        _ => unreachable!(),
    }
}

fn main() {
    primal_network_simplex();
}
