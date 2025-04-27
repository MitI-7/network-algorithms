use network_algorithms::maximum_flow::dinic::Dinic;
use network_algorithms::maximum_flow::graph::Graph;
use network_algorithms::maximum_flow::push_relabel_highest_label::PushRelabelHighestLabel;

fn make_sample_graph() -> Graph<i32> {
    let mut graph = Graph::default();
    graph.add_nodes(4);

    graph.add_directed_edge(0, 1, 2).unwrap();
    graph.add_directed_edge(0, 2, 1).unwrap();
    graph.add_directed_edge(1, 2, 1).unwrap();
    graph.add_directed_edge(1, 3, 1).unwrap();
    graph.add_directed_edge(2, 3, 2).unwrap();

    graph
}

fn dinic() {
    let mut graph = make_sample_graph();
    match Dinic::default().solve(&mut graph, 0, 3, None) {
        Ok(value) => {
            println!("dinic");
            println!("maximum flow:{}", value);
            for edge_id in 0..graph.num_edges() {
                println!("{:?}", graph.get_edge(edge_id).unwrap());
            }
        }
        _ => unreachable!(),
    }
}

fn push_relabel() {
    let mut graph = make_sample_graph();
    match PushRelabelHighestLabel::default().solve(&mut graph, 0, 3, None) {
        Ok(value) => {
            println!("push relabel");
            println!("maximum flow:{}", value);
            for edge_id in 0..graph.num_edges() {
                println!("{:?}", graph.get_edge(edge_id).unwrap());
            }
        }
        _ => unreachable!(),
    }
}

fn main() {
    dinic();
    push_relabel();
}
