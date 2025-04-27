use network_algorithms::maximum_flow::dinic::Dinic;
use network_algorithms::maximum_flow::graph::Graph;

fn main() {
    let mut graph = Graph::default();
    graph.add_nodes(4);

    let edge_ids = vec![
        graph.add_directed_edge(0, 1, 2).unwrap(),
        graph.add_directed_edge(0, 2, 1).unwrap(),
        graph.add_directed_edge(1, 2, 1).unwrap(),
        graph.add_directed_edge(1, 3, 1).unwrap(),
        graph.add_directed_edge(2, 3, 2).unwrap(),
    ];

    match Dinic::default().solve(&mut graph, 0, 3, None) {
        Ok(value) => {
            println!("maximum flow:{}", value);
            for edge_id in edge_ids {
                println!("{:?}", graph.get_edge(edge_id).unwrap());
            }
        }
        _ => unreachable!(),
    }
}
