use network_algorithms::minimum_cost_flow::{
    // CostScalingPushRelabel, CycleCanceling, DualNetworkSimplex, Graph, OutOfKilter, ParametricNetworkSimplex, PrimalDual, PrimalNetworkSimplex,
    CycleCanceling,
    // DualNetworkSimplex,
    Graph,
    OutOfKilter,
    // ParametricNetworkSimplex,
    PrimalDual,
    // PrimalNetworkSimplex,
    SuccessiveShortestPath,
};
use rstest::rstest;
use std::fs::read_to_string;
use std::path::PathBuf;

enum Solver {
    // CostScalingPushRelabel,
    NegativeCostCanceling,
    OutOfKilter,
    PrimalDual,
    SuccessiveShortestPath,

    DualNetworkSimplex,
    ParametricNetworkSimplex,
    PrimalNetworkSimplex,
}

#[rstest]
// #[case::cs(Solver::CostScalingPushRelabel)]
#[case::nc(Solver::NegativeCostCanceling)]
#[case::ok(Solver::OutOfKilter)]
#[case::pd(Solver::PrimalDual)]
#[case::ssp(Solver::SuccessiveShortestPath)]
// #[case::ns_dual(Solver::DualNetworkSimplex)]
// #[case::ns_parametric(Solver::ParametricNetworkSimplex)]
// #[case::ns_primal(Solver::PrimalNetworkSimplex)]
fn minimum_cost_flow(#[files("tests/minimum_cost_flow/*/*.txt")] input_file_path: PathBuf, #[case] solver: Solver) {
    let (mut num_nodes, mut num_edges, mut expected) = (0, 0, "dummy".to_string());
    let mut graph = Graph::<i128>::default();

    read_to_string(&input_file_path).unwrap().split('\n').enumerate().for_each(|(i, line)| {
        let line: Vec<&str> = line.split_whitespace().collect();
        if i == 0 {
            (num_nodes, num_edges, expected) = (line[0].parse::<usize>().unwrap(), line[1].parse::<usize>().unwrap(), line[2].to_string());
            graph.add_nodes(num_nodes);
        } else if i <= num_nodes {
            let b = line[0].parse().unwrap();
            graph.add_supply(i - 1, b);
        } else {
            let (from, to, lower, upper, cost) =
                (line[0].parse().unwrap(), line[1].parse().unwrap(), line[2].parse().unwrap(), line[3].parse().unwrap(), line[4].parse().unwrap());
            assert!(graph.add_directed_edge(from, to, lower, upper, cost).is_some());
        }
    });

    let result = match solver {
        // Solver::CostScalingPushRelabel => CostScalingPushRelabel::default().solve(&mut graph),
        Solver::NegativeCostCanceling => {
            // to slow...
            if input_file_path.to_str().unwrap().contains("LibraryChecker") {
                return;
            }
            CycleCanceling::default().solve(&mut graph)
        }
        Solver::OutOfKilter => {
            // to slow...
            if input_file_path.to_str().unwrap().contains("anti_ssp_00.txt") {
                return;
            }
            OutOfKilter::default().solve(&mut graph)
        }
        Solver::PrimalDual => {
            if input_file_path.to_str().unwrap().contains("anti_ssp_00.txt") {
                return;
            }
            PrimalDual::default().solve(&mut graph)
        }
        Solver::SuccessiveShortestPath => {
            // to slow...
            if input_file_path.to_str().unwrap().contains("anti_ssp_00.txt") {
                return;
            }
            SuccessiveShortestPath::default().solve(&mut graph)
        }
        //
        // Solver::DualNetworkSimplex => DualNetworkSimplex::<i128>::default().solve(&mut graph),
        // Solver::ParametricNetworkSimplex => {
        //     // to slow...
        //     if input_file_path.to_str().unwrap().contains("anti_ssp_00.txt") {
        //         return;
        //     }
        //     ParametricNetworkSimplex::default().solve(&mut graph)
        // }
        // Solver::PrimalNetworkSimplex => PrimalNetworkSimplex::<i128>::default().solve(&mut graph),
        _ => {return;}
    };

    match result {
        Ok(actual) => {
            assert_eq!(actual, expected.parse().unwrap(), "{:?}", input_file_path);
        }
        _ => assert_eq!("infeasible", expected, "{:?}", input_file_path),
    }
    assert_eq!(graph.num_nodes(), num_nodes);
    assert_eq!(graph.num_edges(), num_edges);
}
