use network_algorithms::{ids::NodeId, maximum_flow::prelude::*};
use rstest::rstest;
use std::{fs::read_to_string, path::PathBuf};

enum Solver {
    // CapacityScaling,
    Dinic,
    // EdmondsKarp,
    FordFulkerson,
    PushRelabelFIFO,
    // PushRelabelHighestLabel,
    // ShortestAugmentingPath,
}

fn load_graph(input_file_path: &PathBuf) -> (usize, usize, NodeId, NodeId, i64, MaximumFlowGraph<i64>) {
    let mut graph = MaximumFlowGraph::new();

    let (mut num_nodes, mut num_edges, mut source, mut sink, mut expected) = (0, 0, NodeId(0), NodeId(0), 0);
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
                    NodeId(line[2].parse().unwrap()),
                    NodeId(line[3].parse().unwrap()),
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

    (num_nodes, num_edges, source, sink, expected, graph)
}

impl Solver {
    pub fn should_skip(&self, path: &PathBuf) -> bool {
        let skip_for_libreoj = matches!(self, Solver::FordFulkerson);
        skip_for_libreoj && path.to_str().map_or(false, |s| s.contains("LibreOJ"))
    }

    pub fn run(&self, graph: &MaximumFlowGraph<i64>, s: NodeId, t: NodeId) -> Result<MaxFlowResult<i64>, Status> {
        match self {
            Solver::Dinic => {
                let mut solver = Dinic::new(graph);
                solver.solve(s, t)
            }
            Solver::FordFulkerson => {
                let mut solver = FordFulkerson::new(graph);
                solver.solve(s, t)
            }
            Solver::PushRelabelFIFO => {
                let mut solver = PushRelabelFifo::new(graph);
                solver.solve(s, t)
            }
        }
    }
}

#[rstest]
// #[case::capacity_scaling(Solver::CapacityScaling)]
#[case::dinic(Solver::Dinic)]
// #[case::edmonds_karp(Solver::EdmondsKarp)]
#[case::ford_fulkerson(Solver::FordFulkerson)]
#[case::push_relabel_fifo(Solver::PushRelabelFIFO)]
// #[case::push_relabel_highest_label(Solver::PushRelabelHighestLabel)]
// #[case::shortest_augmenting_path(Solver::ShortestAugmentingPath)]
fn maximum_flow(#[files("tests/maximum_flow/*/*.txt")] path: PathBuf, #[case] solver: Solver) {
    let (num_nodes, num_edges, source, sink, expected, graph) = load_graph(&path);

    if solver.should_skip(&path) {
        return;
    }

    let actual = solver.run(&graph, source, sink).unwrap();
    assert_eq!(graph.num_nodes(), num_nodes);
    assert_eq!(graph.num_edges(), num_edges);
    assert_eq!(actual.objective_value, expected);
}

#[rstest]
// #[case::capacity_scaling(Solver::CapacityScaling)]
#[case::dinic(Solver::Dinic)]
// #[case::edmonds_karp(Solver::EdmondsKarp)]
#[case::ford_fulkerson(Solver::FordFulkerson)]
// #[case::push_relabel_fifo(Solver::PushRelabelFIFO)]
// #[case::push_relabel_highest_label(Solver::PushRelabelHighestLabel)]
// #[case::shortest_augmenting_path(Solver::ShortestAugmentingPath)]
fn maximum_flow_source_eq_sink(#[case] solver: Solver) {
    let mut graph = MaximumFlowGraph::new();
    let nodes = graph.add_nodes(2);
    graph.add_edge(nodes[0], nodes[1], 1);

    let actual = solver.run(&graph, nodes[0], nodes[0]);
    assert_eq!(actual.err().unwrap(), Status::BadInput);
}

#[rstest]
// #[case::capacity_scaling(Solver::CapacityScaling)]
#[case::dinic(Solver::Dinic)]
// #[case::edmonds_karp(Solver::EdmondsKarp)]
#[case::ford_fulkerson(Solver::FordFulkerson)]
// #[case::push_relabel_fifo(Solver::PushRelabelFIFO)]
// #[case::push_relabel_highest_label(Solver::PushRelabelHighestLabel)]
// #[case::shortest_augmenting_path(Solver::ShortestAugmentingPath)]
fn maximum_flow_no_edges(#[case] solver: Solver) {
    let mut graph = MaximumFlowGraph::new();
    let nodes = graph.add_nodes(10);

    let actual = solver.run(&graph, nodes[0], nodes[9]).unwrap();
    assert_eq!(actual.objective_value, 0);
}
