use network_algorithms::maximum_flow::prelude::*;

fn ford_fulkerson_sample() {
    let mut graph = MaximumFlowGraph::new();
    let nodes = graph.add_nodes(6);
    assert_eq!(graph.num_nodes(), 6);
    let mut edges = Vec::new();
    edges.push(graph.add_edge(nodes[0], nodes[1], 3));
    edges.push(graph.add_edge(nodes[0], nodes[2], 3));
    edges.push(graph.add_edge(nodes[1], nodes[2], 2));
    edges.push(graph.add_edge(nodes[1], nodes[3], 3));
    edges.push(graph.add_edge(nodes[2], nodes[4], 2));
    edges.push(graph.add_edge(nodes[3], nodes[4], 4));
    edges.push(graph.add_edge(nodes[3], nodes[5], 2));
    edges.push(graph.add_edge(nodes[4], nodes[5], 3));

    match FordFulkerson::default().solve(&graph, nodes[0], nodes[5], None) {
        Ok(result) => {
            println!("maximum flow:{}", result.objective_value);
            for edge_id in edges {
                println!(
                    "{:?}: {}",
                    graph.get_edge(edge_id),
                    result.flows[edge_id.index()]
                );
            }
            assert_eq!(result.objective_value, 5);
        }
        _ => unreachable!(),
    }
}

// fn push_relabel() {
//     let mut graph = MaximumFlowGraph::<i32>::default();
//     let nodes = graph.add_nodes(4);
//     let mut edges = Vec::new();
//     edges.push(graph.add_directed_edge(nodes[0], nodes[1], 2));
//     edges.push(graph.add_directed_edge(nodes[0], nodes[2], 1));
//     edges.push(graph.add_directed_edge(nodes[1], nodes[2], 1));
//     edges.push(graph.add_directed_edge(nodes[1], nodes[3], 1));
//     edges.push(graph.add_directed_edge(nodes[2], nodes[3], 2));
//
//     match PushRelabelHighestLabel::default()
//         .set_value_only(true)
//         .set_global_relabel_freq(0.5)
//         .solve(&mut graph, nodes[0], nodes[3], None)
//     {
//         Ok(value) => {
//             println!("maximum flow:{}", value);
//             for edge_id in edges {
//                 println!("{:?}", graph.get_edge(edge_id));
//             }
//         }
//         _ => unreachable!(),
//     }
// }

fn main() {
    ford_fulkerson_sample();
    //
    // println!("push relabel");
    // push_relabel();
}
