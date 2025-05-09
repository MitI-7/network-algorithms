use network_algorithms::core::direction::Directed;
use network_algorithms::core::graph::Graph;
use network_algorithms::edge::capacity::CapEdge;
use network_algorithms::maximum_flow::{
    // CapacityScaling, Dinic, EdmondsKarp, FordFulkerson, Graph, PushRelabelFIFO, PushRelabelHighestLabel, ShortestAugmentingPath,
    Dinic,
    // EdmondsKarp,
    // FordFulkerson,
    // Graph,
    // PushRelabelFIFO,
    // PushRelabelHighestLabel,
    // ShortestAugmentingPath,
};
use rstest::rstest;
use std::fs::read_to_string;
use std::path::PathBuf;

enum Solver {
    // CapacityScaling,
    Dinic,
    // EdmondsKarp,
    // FordFulkerson,
    // PushRelabelFIFO,
    // PushRelabelHighestLabel,
    // ShortestAugmentingPath,
}

#[rstest]
// #[case::capacity_scaling(Solver::CapacityScaling)]
#[case::dinic(Solver::Dinic)]
// #[case::edmonds_karp(Solver::EdmondsKarp)]
// #[case::ford_fulkerson(Solver::FordFulkerson)]
// #[case::push_relabel_fifo(Solver::PushRelabelFIFO)]
// #[case::push_relabel_highest_label(Solver::PushRelabelHighestLabel)]
// #[case::shortest_augmenting_path(Solver::ShortestAugmentingPath)]
fn maximum_flow(#[files("tests/maximum_flow/*/*.txt")] input_file_path: PathBuf, #[case] solver: Solver) {
    // let mut graph = Graph::<usize>::default();
    let mut graph: Graph<Directed, (), CapEdge<usize>> = Graph::new();
    let (mut num_nodes, mut num_edges, mut source, mut sink, mut expected) = (0, 0, 0, 0, 0);
    let mut nodes = Vec::new();
    read_to_string(&input_file_path).unwrap().split('\n').enumerate().for_each(|(i, line)| {
        let line: Vec<&str> = line.split_whitespace().collect();
        if i == 0 {
            (num_nodes, num_edges, source, sink, expected) =
                (line[0].parse().unwrap(), line[1].parse().unwrap(), line[2].parse().unwrap(), line[3].parse().unwrap(), line[4].parse().unwrap());
            nodes = graph.add_nodes(num_nodes);
        } else {
            let (from, to, upper) = (line[0].parse::<usize>().unwrap(), line[1].parse::<usize>().unwrap(), line[2].parse::<usize>().unwrap());
            graph.add_directed_edge(nodes[from], nodes[to], CapEdge { flow: 0, upper });
        }
    });

    let actual = match solver {
        // Solver::CapacityScaling => CapacityScaling::default().solve(&mut graph, source, sink, None),
        Solver::Dinic => Dinic::default().solve(&mut graph, source, sink, None),
        // Solver::EdmondsKarp => {
        //     if input_file_path.to_str().unwrap().contains("LibreOJ") {
        //         return;
        //     }
        //     EdmondsKarp::default().solve(&mut graph, source, sink, None)
        // }
        // Solver::FordFulkerson => {
        //     if input_file_path.to_str().unwrap().contains("LibreOJ") {
        //         return;
        //     }
        //     FordFulkerson::default().solve(&mut graph, source, sink, None)
        // }
        // Solver::PushRelabelFIFO => {
        //     if input_file_path.to_str().unwrap().contains("LibreOJ") {
        //         return;
        //     }
        //     PushRelabelFIFO::default().solve(&mut graph, source, sink, None)
        // }
        // Solver::PushRelabelHighestLabel => PushRelabelHighestLabel::default().solve(&mut graph, source, sink, None),
        // Solver::ShortestAugmentingPath => ShortestAugmentingPath::default().solve(&mut graph, source, sink, None),
    };

    assert_eq!(actual.unwrap(), expected);
    assert_eq!(graph.num_nodes(), num_nodes);
    assert_eq!(graph.num_edges(), num_edges);
}
