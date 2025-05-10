// use network_algorithms::algorithms::generalized_maximum_flow::generalized_maximum_flow_graph::Graph;
// use network_algorithms::algorithms::generalized_maximum_flow::highest_gain_path::HighestGainPath;
// use network_algorithms::algorithms::generalized_maximum_flow::primal_dual::PrimalDual;
// use network_algorithms::algorithms::generalized_maximum_flow::primal_dual_push_relabel::PrimalDualPushRelabel;
// use network_algorithms::algorithms::generalized_maximum_flow::status::Status;
// use rstest::rstest;
// use std::fs::read_to_string;
// use std::path::PathBuf;
// 
// enum Solver {
//     HighestGainPath,
//     PrimalDual,
//     PrimalDualPushRelabel,
// }
// 
// #[rstest]
// #[case::highest_gain_path(Solver::HighestGainPath)]
// #[case::primal_dual(Solver::PrimalDual)]
// #[case::primal_dual_push_relabel(Solver::PrimalDualPushRelabel)]
// fn generalized_maximum_flow(#[files("tests/generalized_maximum_flow/*/*.txt")] input_file_path: PathBuf, #[case] solver: Solver) {
//     let epsilon: f64 = 0.01;
//     let mut graph = Graph::default();
//     let mut num_nodes = 0;
//     let mut expected = 0_f64;
//     read_to_string(&input_file_path).unwrap().split('\n').enumerate().for_each(|(i, line)| {
//         let line: Vec<&str> = line.split_whitespace().collect();
//         if i == 0 {
//             (num_nodes, expected) = (line[0].parse::<usize>().unwrap(), line[2].parse::<f64>().unwrap());
//             graph.add_nodes(num_nodes);
//         } else {
//             let (from, to, upper, gain) = (line[0].parse().unwrap(), line[1].parse().unwrap(), line[3].parse().unwrap(), line[4].parse().unwrap());
//             graph.add_directed_edge(from, to, upper, gain);
//         }
//     });
//     let sink = num_nodes - 1;
// 
//     let status = match solver {
//         Solver::HighestGainPath => HighestGainPath::new(epsilon).solve(0, sink, &mut graph),
//         Solver::PrimalDual => PrimalDual::new(epsilon).solve(0, sink, &mut graph),
//         Solver::PrimalDualPushRelabel => PrimalDualPushRelabel::new(epsilon).solve(0, sink, &mut graph),
//     };
//     let actual = graph.maximum_flow(sink);
// 
//     assert_eq!(status, Status::Optimal);
// 
//     if expected == 0.0 {
//         assert!(actual < 0.001);
//     } else {
//         assert!(expected * (1.0 - epsilon) <= actual && actual <= expected, "{}/{}({:?})", actual, expected, input_file_path);
//     }
//     assert_eq!(graph.num_nodes(), num_nodes);
//     // assert_eq!(graph.num_edges(), num_edges);
// }
