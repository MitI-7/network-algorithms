use core::ops::{Div, DivAssign, Mul, MulAssign};
use network_algorithms::core::ids::NodeId;
use network_algorithms::maximum_flow::{
    CapacityScaling, Dinic, EdmondsKarp, FlowNum, FordFulkerson, MaximumFlowGraph, MaximumFlowSolver, PushRelabelFIFO, PushRelabelHighestLabel, ShortestAugmentingPath,
    Status,
};
use network_algorithms::traits::One;
use network_algorithms::traits::Zero;
use rstest::rstest;
use std::fmt::Debug;
use std::fs::read_to_string;
use std::path::PathBuf;
use std::str::FromStr;

enum Solver {
    CapacityScaling,
    Dinic,
    EdmondsKarp,
    FordFulkerson,
    PushRelabelFIFO,
    PushRelabelHighestLabel,
    ShortestAugmentingPath,
}

fn load_graph<F: Copy + Zero + FromStr>(input_file_path: &PathBuf) -> (usize, usize, NodeId, NodeId, F, MaximumFlowGraph<F>)
where
    <F as FromStr>::Err: Debug,
{
    let mut graph = MaximumFlowGraph::<F>::new();
    let (mut num_nodes, mut num_edges, mut source, mut sink, mut expected) = (0, 0, NodeId(0), NodeId(0), F::zero());
    let mut nodes = Vec::new();

    read_to_string(&input_file_path).unwrap().split('\n').enumerate().for_each(|(i, line)| {
        let line: Vec<&str> = line.split_whitespace().collect();
        if i == 0 {
            (num_nodes, num_edges, source, sink, expected) =
                (line[0].parse().unwrap(), line[1].parse().unwrap(), NodeId(line[2].parse().unwrap()), NodeId(line[3].parse().unwrap()), line[4].parse().unwrap());
            nodes = graph.add_nodes(num_nodes);
        } else {
            let (from, to, upper) = (line[0].parse::<usize>().unwrap(), line[1].parse::<usize>().unwrap(), line[2].parse::<F>().unwrap());
            graph.add_directed_edge(nodes[from], nodes[to], upper);
        }
    });

    (num_nodes, num_edges, source, sink, expected, graph)
}

impl Solver {
    pub fn should_skip(&self, path: &PathBuf) -> bool {
        let skip_for_libreoj = matches!(self, Solver::EdmondsKarp | Solver::FordFulkerson | Solver::PushRelabelFIFO);
        skip_for_libreoj && path.to_str().map_or(false, |s| s.contains("LibreOJ"))
    }

    pub fn build<Flow: FlowNum + Default + One + Mul<Output = Flow> + MulAssign + Div<Output = Flow> + DivAssign + 'static>(&self) -> Box<dyn MaximumFlowSolver<Flow>> {
        match self {
            Solver::CapacityScaling => Box::new(CapacityScaling::default()),
            Solver::Dinic => Box::new(Dinic::default()),
            Solver::EdmondsKarp => Box::new(EdmondsKarp::default()),
            Solver::FordFulkerson => Box::new(FordFulkerson::default()),
            Solver::PushRelabelFIFO => Box::new(PushRelabelFIFO::default()),
            Solver::PushRelabelHighestLabel => Box::new(PushRelabelHighestLabel::default()),
            Solver::ShortestAugmentingPath => Box::new(ShortestAugmentingPath::default()),
        }
    }
}

#[rstest]
#[case::capacity_scaling(Solver::CapacityScaling)]
#[case::dinic(Solver::Dinic)]
#[case::edmonds_karp(Solver::EdmondsKarp)]
#[case::ford_fulkerson(Solver::FordFulkerson)]
#[case::push_relabel_fifo(Solver::PushRelabelFIFO)]
#[case::push_relabel_highest_label(Solver::PushRelabelHighestLabel)]
#[case::shortest_augmenting_path(Solver::ShortestAugmentingPath)]
fn maximum_flow(#[files("tests/maximum_flow/*/*.txt")] path: PathBuf, #[case] solver: Solver) {
    let (num_nodes, num_edges, source, sink, expected, mut graph) = load_graph::<i64>(&path);

    if solver.should_skip(&path) {
        return;
    }
    let mut solver_impl = solver.build();
    let actual = solver_impl.solve(&mut graph, source, sink, None);
    assert_eq!(actual.unwrap(), expected);
    assert_eq!(graph.num_nodes(), num_nodes);
    assert_eq!(graph.num_edges(), num_edges);
}

#[rstest]
#[case::capacity_scaling(Solver::CapacityScaling)]
#[case::dinic(Solver::Dinic)]
#[case::edmonds_karp(Solver::EdmondsKarp)]
#[case::ford_fulkerson(Solver::FordFulkerson)]
#[case::push_relabel_fifo(Solver::PushRelabelFIFO)]
#[case::push_relabel_highest_label(Solver::PushRelabelHighestLabel)]
#[case::shortest_augmenting_path(Solver::ShortestAugmentingPath)]
fn source_eq_sink(#[case] solver: Solver) {
    let mut graph = MaximumFlowGraph::<usize>::new();
    let nodes = graph.add_nodes(2);
    graph.add_directed_edge(nodes[0], nodes[1], 1);

    let mut solver_impl = solver.build();
    let actual = solver_impl.solve(&mut graph, nodes[0], nodes[0], None);
    assert_eq!(actual.err().unwrap(), Status::BadInput);
}

#[rstest]
#[case::capacity_scaling(Solver::CapacityScaling)]
#[case::dinic(Solver::Dinic)]
#[case::edmonds_karp(Solver::EdmondsKarp)]
#[case::ford_fulkerson(Solver::FordFulkerson)]
#[case::push_relabel_fifo(Solver::PushRelabelFIFO)]
#[case::push_relabel_highest_label(Solver::PushRelabelHighestLabel)]
#[case::shortest_augmenting_path(Solver::ShortestAugmentingPath)]
fn no_edges(#[case] solver: Solver) {
    let mut graph = MaximumFlowGraph::<usize>::new();
    let nodes = graph.add_nodes(10);

    let mut solver_impl = solver.build();
    let actual = solver_impl.solve(&mut graph, nodes[0], nodes[9], None);
    assert_eq!(actual.unwrap(), 0);
}
