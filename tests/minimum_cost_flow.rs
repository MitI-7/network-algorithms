use network_algorithms::{minimum_cost_flow::MinimumCostFlowNum, minimum_cost_flow::prelude::*};
use rstest::rstest;
use std::{fmt::Debug, fs::read_to_string, path::PathBuf};

enum Solver {
    // CostScalingPushRelabel,
    // NegativeCostCanceling,
    // OutOfKilter,
    PrimalDual,
    SuccessiveShortestPath,
    // DualNetworkSimplex,
    // ParametricNetworkSimplex,
    // PrimalNetworkSimplex,
}

impl Solver {
    pub fn should_skip(&self, path: &PathBuf) -> bool {
        // let skip_for_lib = matches!(self, Solver::NegativeCostCanceling);
        // let a = skip_for_lib && path.to_str().map_or(false, |s| s.contains("LibraryChecker"));
        let skip_for_anti =
            // matches!(self, Solver::OutOfKilter | Solver::PrimalDual | Solver::SuccessiveShortestPath | Solver::ParametricNetworkSimplex);
            matches!(self, Solver::PrimalDual | Solver::SuccessiveShortestPath);
        let b = skip_for_anti && path.to_str().map_or(false, |s| s.contains("anti_ssp_00"));
        // a || b
        b
    }

    // pub fn build<F, P>(&self) -> Box<dyn MinimumCostFlowSolver<F>>
    pub fn build<F>(&self) -> Box<dyn MinimumCostFlowSolver<F>>
    where
        F: MinimumCostFlowNum
            + TryFrom<usize>
            + std::ops::MulAssign
            + std::ops::Div<Output = F>
            + std::ops::DivAssign
            + Default
            + 'static,
        // P: Default + PivotRule<F> + 'static,
        <F as TryFrom<usize>>::Error: Debug,
    {
        match self {
            // Solver::CostScalingPushRelabel => Box::new(CostScalingPushRelabel::default()),
            // Solver::NegativeCostCanceling => Box::new(CycleCanceling::default()),
            // Solver::OutOfKilter => Box::new(OutOfKilter::default()),
            Solver::PrimalDual => Box::new(PrimalDual::default()),
            Solver::SuccessiveShortestPath => Box::new(SuccessiveShortestPath::default()),
            // Solver::DualNetworkSimplex => Box::new(DualNetworkSimplex::<F, P>::default()),
            // Solver::ParametricNetworkSimplex => Box::new(ParametricNetworkSimplex::default()),
            // Solver::PrimalNetworkSimplex => Box::new(PrimalNetworkSimplex::<F, P>::default()),
        }
    }
}

#[rstest]
// #[case::cs(Solver::CostScalingPushRelabel)]
// #[case::nc(Solver::NegativeCostCanceling)]
// #[case::ok(Solver::OutOfKilter)]
#[case::pd(Solver::PrimalDual)]
#[case::ssp(Solver::SuccessiveShortestPath)]
// #[case::ns_dual(Solver::DualNetworkSimplex)]
// #[case::ns_parametric(Solver::ParametricNetworkSimplex)]
// #[case::ns_primal(Solver::PrimalNetworkSimplex)]
fn minimum_cost_flow(
    #[files("tests/minimum_cost_flow/*/*.txt")] path: PathBuf,
    #[case] solver: Solver,
) {
    if solver.should_skip(&path) {
        return;
    }

    let (mut num_nodes, mut num_edges, mut expected) = (0, 0, "dummy".to_string());
    let mut graph = MinimumCostFlowGraph::<i128>::new();
    let mut nodes = Vec::new();

    read_to_string(&path)
        .unwrap()
        .split('\n')
        .enumerate()
        .for_each(|(i, line)| {
            let line: Vec<&str> = line.split_whitespace().collect();
            if i == 0 {
                (num_nodes, num_edges, expected) = (
                    line[0].parse::<usize>().unwrap(),
                    line[1].parse::<usize>().unwrap(),
                    line[2].to_string(),
                );
                nodes = graph.add_nodes(num_nodes);
            } else if i <= num_nodes {
                let b = line[0].parse().unwrap();
                graph.get_node_mut(nodes[i - 1]).data.b = b;
            } else {
                let (from, to, lower, upper, cost) = (
                    line[0].parse::<usize>().unwrap(),
                    line[1].parse::<usize>().unwrap(),
                    line[2].parse().unwrap(),
                    line[3].parse().unwrap(),
                    line[4].parse().unwrap(),
                );
                graph.add_edge(nodes[from], nodes[to], lower, upper, cost);
            }
        });

    // let mut solver_impl = solver.build::<_, BlockSearchPivotRule<_>>();
    let mut solver_impl = solver.build::<_>();
    let actual = solver_impl.solve(&mut graph);

    match actual {
        Ok(actual) => {
            assert_eq!(
                actual.objective_value,
                expected.parse().unwrap(),
                "{:?}",
                path
            );
        }
        _ => assert_eq!("infeasible", expected, "{:?}", path),
    }
    assert_eq!(graph.num_nodes(), num_nodes);
    assert_eq!(graph.num_edges(), num_edges);
}
//
// #[rstest]
// #[case::cs(Solver::CostScalingPushRelabel)]
// #[case::nc(Solver::NegativeCostCanceling)]
// #[case::ok(Solver::OutOfKilter)]
// #[case::pd(Solver::PrimalDual)]
// #[case::ssp(Solver::SuccessiveShortestPath)]
// #[case::ns_dual(Solver::DualNetworkSimplex)]
// #[case::ns_parametric(Solver::ParametricNetworkSimplex)]
// #[case::ns_primal(Solver::PrimalNetworkSimplex)]
// fn minimum_cost_flow_unbalance(#[case] solver: Solver) {
//     let mut graph = MinimumCostFlowGraph::<i32>::new();
//     let nodes = graph.add_nodes(2);
//     graph.add_directed_edge(nodes[0], nodes[1], 0, 1, 1);
//
//     graph.nodes[0].b = 1;
//     graph.nodes[1].b = 1;
//
//     let mut solver_impl = solver.build::<_, BlockSearchPivotRule<_>>();
//     let actual = solver_impl.solve(&mut graph);
//     assert_eq!(actual.err().unwrap(), Status::Unbalanced);
// }
//
// #[rstest]
// // #[case::cs(Solver::CostScalingPushRelabel)]
// #[case::nc(Solver::NegativeCostCanceling)]
// #[case::ok(Solver::OutOfKilter)]
// #[case::pd(Solver::PrimalDual)]
// #[case::ssp(Solver::SuccessiveShortestPath)]
// #[case::ns_dual(Solver::DualNetworkSimplex)]
// #[case::ns_parametric(Solver::ParametricNetworkSimplex)]
// #[case::ns_primal(Solver::PrimalNetworkSimplex)]
// fn minimum_cost_flow_no_edges(#[case] solver: Solver) {
//     let mut graph = MinimumCostFlowGraph::<i32>::new();
//     let _nodes = graph.add_nodes(2);
//     graph.nodes[0].b = 1;
//     graph.nodes[1].b = -1;
//
//     let mut solver_impl = solver.build::<_, BlockSearchPivotRule<_>>();
//     let actual = solver_impl.solve(&mut graph);
//     assert_eq!(actual.err().unwrap(), Status::Infeasible);
// }
//
// #[rstest]
// #[case::cs(Solver::CostScalingPushRelabel)]
// #[case::nc(Solver::NegativeCostCanceling)]
// #[case::ok(Solver::OutOfKilter)]
// #[case::pd(Solver::PrimalDual)]
// #[case::ssp(Solver::SuccessiveShortestPath)]
// #[case::ns_dual(Solver::DualNetworkSimplex)]
// #[case::ns_parametric(Solver::ParametricNetworkSimplex)]
// #[case::ns_primal(Solver::PrimalNetworkSimplex)]
// fn minimum_cost_flow_no_nodes(#[case] solver: Solver) {
//     let mut graph = MinimumCostFlowGraph::<i32>::new();
//     let mut solver_impl = solver.build::<_, BlockSearchPivotRule<_>>();
//     let actual = solver_impl.solve(&mut graph);
//     assert_eq!(actual.unwrap(), 0);
// }
