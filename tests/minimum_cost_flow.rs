use network_algorithms::core::numeric::CostNum;
use network_algorithms::ids::EdgeId;
use network_algorithms::minimum_cost_flow::prelude::*;
use rstest::rstest;
use rstest_reuse::{self, *};
use std::{fmt::Debug, fs::read_to_string, path::Path, path::PathBuf};

#[template]
#[rstest]
#[case(Solver::CostScalingPushRelabel)]
#[case(Solver::CycleCanceling)]
#[case(Solver::OutOfKilter)]
#[case(Solver::PrimalDual)]
#[case(Solver::SuccessiveShortestPath)]
#[case(Solver::DualNetworkSimplex)]
#[case(Solver::ParametricNetworkSimplex)]
#[case(Solver::PrimalNetworkSimplex)]
fn all_solvers(#[case] solver: Solver) {}

enum Solver {
    CostScalingPushRelabel,
    CycleCanceling,
    OutOfKilter,
    PrimalDual,
    SuccessiveShortestPath,
    DualNetworkSimplex,
    ParametricNetworkSimplex,
    PrimalNetworkSimplex,
}

impl Solver {
    pub fn skip(&self, path: &Path) -> bool {
        let skip_for_lib = matches!(self, Solver::CycleCanceling);
        let a = skip_for_lib && path.to_str().map_or(false, |s| s.contains("LibraryChecker"));
        let skip_for_anti = matches!(
            self,
            Solver::OutOfKilter
                | Solver::PrimalDual
                | Solver::SuccessiveShortestPath
                | Solver::ParametricNetworkSimplex
        );
        let b = skip_for_anti && path.to_str().map_or(false, |s| s.contains("anti_ssp_00"));
        a || b
    }
    pub fn get(&self, graph: &MinimumCostFlowGraph<i128>) -> Box<dyn MinimumCostFlowSolver<i128>> {
        match self {
            Solver::CostScalingPushRelabel => {
                Box::new(<CostScalingPushRelabel<i128> as MinimumCostFlowSolver<i128>>::new(graph))
            }
            Solver::CycleCanceling => Box::new(<CycleCanceling<i128> as MinimumCostFlowSolver<i128>>::new(graph)),
            Solver::OutOfKilter => Box::new(<OutOfKilter<i128> as MinimumCostFlowSolver<i128>>::new(graph)),
            Solver::PrimalDual => Box::new(<PrimalDual<i128> as MinimumCostFlowSolver<i128>>::new(graph)),
            Solver::SuccessiveShortestPath => {
                Box::new(<SuccessiveShortestPath<i128> as MinimumCostFlowSolver<i128>>::new(graph))
            }
            Solver::DualNetworkSimplex => {
                Box::new(<DualNetworkSimplex<i128> as MinimumCostFlowSolver<i128>>::new(graph))
            }
            Solver::ParametricNetworkSimplex => {
                Box::new(<ParametricNetworkSimplex<i128> as MinimumCostFlowSolver<i128>>::new(graph))
            }
            Solver::PrimalNetworkSimplex => {
                Box::new(<PrimalNetworkSimplex<i128> as MinimumCostFlowSolver<i128>>::new(graph))
            }
        }
    }
}

fn check_optimality<F: CostNum + Debug>(
    graph: &MinimumCostFlowGraph<F>,
    edges: &[EdgeId],
    flows: &[F],
    potentials: &[F],
) -> bool {
    let mut ok = true;
    for &edge_id in edges {
        let edge = graph.get_edge(edge_id).unwrap();
        let (u, v) = (edge.u, edge.v);
        let f = flows[edge_id.index()];

        // Feasibility (capacity)
        if f < edge.data.lower || f > edge.data.upper {
            return false;
        }

        if edge.data.lower == edge.data.upper {
            continue;
        }

        let r = edge.data.cost - potentials[u.index()] + potentials[v.index()];

        // Complementary slackness (optimality witness by potentials)
        ok &= if f == edge.data.lower {
            r >= F::zero()
        } else if edge.data.lower < f && f < edge.data.upper {
            r == F::zero()
        } else {
            r <= F::zero()
        };
    }
    ok
}

fn read_graph(path: &Path) -> (MinimumCostFlowGraph<i128>, Vec<EdgeId>, String) {
    let (mut num_nodes, mut num_edges, mut expected) = (0, 0, "dummy".to_string());
    let mut graph = MinimumCostFlowGraph::<i128>::default();
    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    read_to_string(&path)
        .unwrap()
        .split('\n')
        .enumerate()
        .for_each(|(i, line)| {
            let line: Vec<&str> = line.split_whitespace().collect();
            if i == 0 {
                (num_nodes, num_edges, expected) =
                    (line[0].parse::<usize>().unwrap(), line[1].parse::<usize>().unwrap(), line[2].to_string());
                nodes = graph.add_nodes(num_nodes);
            } else if i <= num_nodes {
                let b = line[0].parse().unwrap();
                graph.get_node_mut(nodes[i - 1]).unwrap().data.b = b;
            } else {
                let (from, to, lower, upper, cost) = (
                    line[0].parse::<usize>().unwrap(),
                    line[1].parse::<usize>().unwrap(),
                    line[2].parse().unwrap(),
                    line[3].parse().unwrap(),
                    line[4].parse().unwrap(),
                );
                edges.push(graph.add_edge(nodes[from], nodes[to], lower, upper, cost).unwrap());
            }
        });

    (graph, edges, expected)
}

#[apply(all_solvers)]
fn minimum_cost_flow(#[files("tests/minimum_cost_flow/*/*.txt")] path: PathBuf, #[case] solver: Solver) {
    if solver.skip(&path) {
        return;
    }
    let (graph, edges, expected) = read_graph(&path);
    let mut s = solver.get(&graph);
    let actual = s.solve();

    match actual {
        Ok(actual) => {
            let flows = s.flows();
            let potentials = s.potentials();
            assert!(check_optimality(&graph, &edges, &flows, &potentials));
            assert_eq!(actual, expected.parse().unwrap(), "{:?}", path);
        }
        _ => assert_eq!("infeasible", expected, "{:?}", path),
    }
}

#[apply(all_solvers)]
fn minimum_cost_flow_unbalance(#[case] solver: Solver) {
    let mut graph = MinimumCostFlowGraph::<i128>::default();
    let nodes = graph.add_nodes(2);
    graph.add_edge(nodes[0], nodes[1], 0, 1, 1);
    graph.get_node_mut(nodes[0]).unwrap().data.b = 1;
    graph.get_node_mut(nodes[1]).unwrap().data.b = 1;

    let actual = solver.get(&graph).solve();
    assert_eq!(actual.err().unwrap(), Status::Unbalanced);
}

#[apply(all_solvers)]
fn minimum_cost_flow_no_edges(#[case] solver: Solver) {
    let mut graph = MinimumCostFlowGraph::<i128>::default();
    let nodes = graph.add_nodes(2);
    graph.get_node_mut(nodes[0]).unwrap().data.b = 1;
    graph.get_node_mut(nodes[1]).unwrap().data.b = -1;

    let actual = solver.get(&graph).solve();
    assert_eq!(actual.err().unwrap(), Status::Infeasible);
}

#[apply(all_solvers)]
fn minimum_cost_flow_no_nodes(#[case] solver: Solver) {
    let graph = MinimumCostFlowGraph::<i128>::default();
    let actual = solver.get(&graph).solve();
    assert_eq!(actual.unwrap(), 0);
}
