use network_algorithms::{ids::NodeId, algorithms::maximum_flow::prelude::*};
use rstest::rstest;
use rstest_reuse::*;
use std::{fs::read_to_string, path::PathBuf};

#[template]
#[rstest]
#[case::capacity_scaling(Solver::CapacityScaling)]
#[case::dinic(Solver::Dinic)]
#[case::edmonds_karp(Solver::EdmondsKarp)]
#[case::ford_fulkerson(Solver::FordFulkerson)]
#[case::push_relabel_fifo(Solver::PushRelabelFIFO)]
#[case::push_relabel_highest_label(Solver::PushRelabelHighestLabel)]
#[case::shortest_augmenting_path(Solver::ShortestAugmentingPath)]
fn all_solvers(#[case] solver: Solver) {}

enum Solver {
    CapacityScaling,
    Dinic,
    EdmondsKarp,
    FordFulkerson,
    PushRelabelFIFO,
    PushRelabelHighestLabel,
    ShortestAugmentingPath,
}

impl Solver {
    pub fn skip(&self, path: &PathBuf) -> bool {
        let skip_for_libreoj = matches!(self, Solver::EdmondsKarp | Solver::FordFulkerson | Solver::PushRelabelFIFO);
        skip_for_libreoj && path.to_str().map_or(false, |s| s.contains("LibreOJ"))
    }

    pub fn get(&self, graph: &MaximumFlowGraph<i64>) -> Box<dyn MaximumFlowSolver<i64>> {
        match self {
            Solver::CapacityScaling => Box::new(<CapacityScaling<i64> as MaximumFlowSolver<i64>>::new(graph)),
            Solver::Dinic => Box::new(<Dinic<i64> as MaximumFlowSolver<i64>>::new(graph)),
            Solver::EdmondsKarp => Box::new(<EdmondsKarp<i64> as MaximumFlowSolver<i64>>::new(graph)),
            Solver::FordFulkerson => Box::new(<FordFulkerson<i64> as MaximumFlowSolver<i64>>::new(graph)),
            Solver::PushRelabelFIFO => Box::new(<PushRelabelFifo<i64> as MaximumFlowSolver<i64>>::new(graph)),
            Solver::ShortestAugmentingPath => {
                Box::new(<ShortestAugmentingPath<i64> as MaximumFlowSolver<i64>>::new(graph))
            }
            Solver::PushRelabelHighestLabel => {
                Box::new(<PushRelabelHighestLabel<i64> as MaximumFlowSolver<i64>>::new(graph))
            }
        }
    }
}

fn load_graph(input_file_path: &PathBuf) -> (NodeId, NodeId, i64, MaximumFlowGraph<i64>) {
    let mut graph = MaximumFlowGraph::default();

    let (mut num_nodes, mut num_edges, mut source, mut sink, mut expected) = (0, 0, 0, 0, 0);
    let mut nodes = Vec::new();

    read_to_string(&input_file_path)
        .unwrap()
        .split('\n')
        .enumerate()
        .for_each(|(i, line)| {
            let line: Vec<&str> = line.split_whitespace().collect();
            if i == 0 {
                (num_nodes, num_edges, source, sink, expected) = (
                    line[0].parse().unwrap(),
                    line[1].parse().unwrap(),
                    line[2].parse().unwrap(),
                    line[3].parse().unwrap(),
                    line[4].parse().unwrap(),
                );
                nodes = graph.add_nodes(num_nodes);
            } else {
                let (from, to, upper) = (
                    line[0].parse::<usize>().unwrap(),
                    line[1].parse::<usize>().unwrap(),
                    line[2].parse::<i64>().unwrap(),
                );
                graph.add_edge(nodes[from], nodes[to], upper);
            }
        });

    (nodes[source], nodes[sink], expected, graph)
}

fn check(graph: &MaximumFlowGraph<i64>, reach: &Vec<bool>) -> i64 {
    let mut o = 0;
    for e in graph.edges() {
        if reach[e.u.index()] && !reach[e.v.index()] {
            o += e.data.upper;
        }
    }

    o
}

#[apply(all_solvers)]
fn maximum_flow(#[files("tests/maximum_flow/*/*.txt")] path: PathBuf, #[case] solver: Solver) {
    if solver.skip(&path) {
        return;
    }

    let (source, sink, expected, graph) = load_graph(&path);
    let mut s = solver.get(&graph);
    let objective_value = s.solve(source, sink).unwrap();
    assert_eq!(objective_value, expected);

    let flows = s.flows();
    let reach = s.minimum_cut().unwrap();
    assert_eq!(check(&graph, &reach), expected);

    for (edge_id, e) in graph.edges().enumerate() {
        if reach[e.u.index()] && !reach[e.v.index()] {
            assert_eq!(flows[edge_id], e.data.upper);
        }
    }
}

#[apply(all_solvers)]
fn maximum_flow_source_eq_sink(#[case] solver: Solver) {
    let mut graph = MaximumFlowGraph::default();
    let nodes = graph.add_nodes(2);
    graph.add_edge(nodes[0], nodes[1], 1);

    let actual = solver.get(&graph).solve(nodes[0], nodes[0]);
    let expected = MaximumFlowError::InvalidTerminal { source: nodes[0], sink: nodes[0], num_nodes: 2 };
    assert_eq!(actual.err().unwrap(), expected);
}

#[apply(all_solvers)]
fn maximum_flow_no_edges(#[case] solver: Solver) {
    let mut graph = MaximumFlowGraph::default();
    let nodes = graph.add_nodes(10);

    let actual = solver.get(&graph).solve(nodes[0], nodes[9]);
    let expected = 0;
    assert_eq!(actual.unwrap(), expected);
}
