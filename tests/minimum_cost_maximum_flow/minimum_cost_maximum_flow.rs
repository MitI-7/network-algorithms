// use network_algorithms::minimum_cost_flow::dual_network_simplex::DualNetworkSimplex;
// use network_algorithms::minimum_cost_flow::graph::Graph;
// use network_algorithms::minimum_cost_flow::network_simplex_pivot_rules::{BlockSearchPivotRule, PivotRule};
// use network_algorithms::minimum_cost_flow::parametric_network_simplex::ParametricNetworkSimplex;
// use network_algorithms::minimum_cost_flow::primal_dual::PrimalDual;
// use network_algorithms::minimum_cost_flow::primal_network_simplex::PrimalNetworkSimplex;
// use network_algorithms::minimum_cost_flow::status::Status;
// use network_algorithms::minimum_cost_flow::successive_shortest_path::SuccessiveShortestPath;
// use rstest::rstest;
// use std::any::TypeId;
// use std::fs::read_to_string;
// use std::path::PathBuf;
//
// #[rstest]
// #[case::ns_pr(TypeId::of::<PrimalNetworkSimplex<i128>>())]
// // #[case::ns_du(TypeId::of::<DualNetworkSimplex<i128>>())]
// // #[case::ns_pa(TypeId::of::<ParametricNetworkSimplex<i128>>())]
// fn minimum_cost_maximum_flow(#[files("tests/minimum_cost_flow/minimum_cost_maximum_flow/*/*.txt")] input_file_path: PathBuf, #[case] solver_type: TypeId) {
//     let (mut num_nodes, mut num_edges, mut expected_flow, mut expected_cost) = (0, 0, 0, 0);
//     let mut graph = Graph::<i128>::default();
//
//     read_to_string(&input_file_path).unwrap().split('\n').enumerate().for_each(|(i, line)| {
//         let line: Vec<&str> = line.split_whitespace().collect();
//         if i == 0 {
//             (num_nodes, num_edges, expected_flow, expected_cost) = (line[0].parse::<usize>().unwrap(), line[1].parse::<usize>().unwrap(), line[2].parse().unwrap(), line[3].parse().unwrap());
//             graph.add_nodes(num_nodes);
//         } else {
//             let (from, to, upper, cost) = (line[0].parse().unwrap(), line[1].parse().unwrap(), line[2].parse().unwrap(), line[3].parse().unwrap());
//             assert!(graph.add_directed_edge(from, to, 0, upper, cost).is_some());
//         }
//     });
//
//     let result = if solver_type == TypeId::of::<PrimalNetworkSimplex<i128>>() {
//         PrimalNetworkSimplex::default().solve_max_flow_with_min_cost(0, num_nodes - 1, &mut BlockSearchPivotRule::new(num_edges), &mut graph)
//     // } else if solver_type == TypeId::of::<DualNetworkSimplex<i128>>() {
//     //     let mut solver = DualNetworkSimplex::new(&mut st);
//     //     solver.solve_max_flow_with_min_cost(0, num_nodes - 1, &mut BlockSearchPivotRule::new(num_edges))
//     // } else if solver_type == TypeId::of::<ParametricNetworkSimplex<i128>>() {
//     //     let mut solver = ParametricNetworkSimplex::new(&mut st);
//     //     solver.solve_max_flow_with_min_cost(0, num_nodes - 1)
//     } else {
//         unreachable!()
//     };
//
//     println!("{:?}, {}", result.0, result.1);
//
//     match result {
//         (Status::Optimal, maximum_flow) => {
//             assert_eq!(graph.minimum_cost(), expected_cost);
//             assert_eq!(maximum_flow, expected_flow);
//         }
//         _ => unreachable!(),
//     }
//     // assert_eq!(st.num_nodes(), num_nodes);
//     // assert_eq!(st.num_edges(), num_edges);
// }
