// use network_algorithms::minimum_cost_flow::network_simplex_pivot_rules::BestEligibleArcPivotRule;
// use network_algorithms::minimum_cost_flow::{Graph, PrimalNetworkSimplex};
//
// fn primal_network_simplex() {
//     let mut graph = Graph::default();
//     graph.add_nodes(4);
//
//     let edge_ids = vec![
//         graph.add_directed_edge(0, 1, 0, 2, 1).unwrap(),
//         graph.add_directed_edge(0, 2, 0, 1, 2).unwrap(),
//         graph.add_directed_edge(1, 2, 0, 1, 1).unwrap(),
//         graph.add_directed_edge(1, 3, 0, 1, 3).unwrap(),
//         graph.add_directed_edge(2, 3, 0, 2, 1).unwrap(),
//     ];
//
//     graph.add_supply(0, 2);
//     graph.add_supply(3, -2);
//
//     let pivot = BestEligibleArcPivotRule::default();
//     match PrimalNetworkSimplex::<i32>::default().set_pivot(pivot).solve(&mut graph) {
//         Ok(value) => {
//             println!("minimum cost:{}", value);
//             for edge_id in edge_ids {
//                 println!("{:?}", graph.get_edge(edge_id).unwrap());
//             }
//         }
//         _ => unreachable!(),
//     }
// }
//
// fn main() {
//     primal_network_simplex();
// }

fn main() {

}