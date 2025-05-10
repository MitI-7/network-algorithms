use network_algorithms::maximum_flow::{Dinic, MaximumFlowGraph, PushRelabelHighestLabel};

fn dinic_sample() {
    let mut graph = MaximumFlowGraph::<i32>::default();
    let nodes = graph.add_nodes(4);
    let mut edges = Vec::new();
    edges.push(graph.add_directed_edge(nodes[0], nodes[1], 2));
    edges.push(graph.add_directed_edge(nodes[0], nodes[2], 1));
    edges.push(graph.add_directed_edge(nodes[1], nodes[2], 1));
    edges.push(graph.add_directed_edge(nodes[1], nodes[3], 1));
    edges.push(graph.add_directed_edge(nodes[2], nodes[3], 2));

    match Dinic::default().solve(&mut graph, nodes[0], nodes[3], None) {
        Ok(value) => {
            println!("maximum flow:{}", value);
            for edge_id in edges {
                println!("{:?}", graph.get_edge(edge_id));
            }
        }
        _ => unreachable!(),
    }
}

fn push_relabel() {
    let mut graph = MaximumFlowGraph::<i32>::default();
    let nodes = graph.add_nodes(4);
    let mut edges = Vec::new();
    edges.push(graph.add_directed_edge(nodes[0], nodes[1], 2));
    edges.push(graph.add_directed_edge(nodes[0], nodes[2], 1));
    edges.push(graph.add_directed_edge(nodes[1], nodes[2], 1));
    edges.push(graph.add_directed_edge(nodes[1], nodes[3], 1));
    edges.push(graph.add_directed_edge(nodes[2], nodes[3], 2));

    match PushRelabelHighestLabel::default()
        .set_value_only(true)
        .set_global_relabel_freq(0.5)
        .solve(&mut graph, nodes[0], nodes[3], None)
    {
        Ok(value) => {
            println!("maximum flow:{}", value);
            for edge_id in edges {
                println!("{:?}", graph.get_edge(edge_id));
            }
        }
        _ => unreachable!(),
    }
}

fn main() {
    println!("dinic");
    dinic_sample();

    println!("push relabel");
    push_relabel();
}
