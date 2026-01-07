use network_algorithms::ids::{EdgeId, NodeId};
use network_algorithms::maximum_flow::prelude::*;

fn make_sample_graph() -> (Vec<NodeId>, Vec<EdgeId>, MaximumFlowGraph<i32>) {
    let mut graph = MaximumFlowGraph::default();
    let nodes = graph.add_nodes(6);
    let mut edges = Vec::new();
    edges.push(graph.add_edge(nodes[0], nodes[1], 3).unwrap());
    edges.push(graph.add_edge(nodes[0], nodes[2], 3).unwrap());
    edges.push(graph.add_edge(nodes[1], nodes[2], 2).unwrap());
    edges.push(graph.add_edge(nodes[1], nodes[3], 3).unwrap());
    edges.push(graph.add_edge(nodes[2], nodes[4], 2).unwrap());
    edges.push(graph.add_edge(nodes[3], nodes[4], 4).unwrap());
    edges.push(graph.add_edge(nodes[3], nodes[5], 2).unwrap());
    edges.push(graph.add_edge(nodes[4], nodes[5], 3).unwrap());

    (nodes, edges, graph)
}

fn ford_fulkerson() {
    let (nodes, edges, graph) = make_sample_graph();
    match FordFulkerson::new(&graph).maximum_flow(nodes[0], nodes[5]) {
        Ok(result) => {
            println!("maximum flow:{}", result.objective_value);
            for edge_id in edges {
                println!("{:?}: {}", graph.get_edge(edge_id), result.flows[edge_id.index()]);
            }
            assert_eq!(result.objective_value, 5);
        }
        _ => unreachable!(),
    }
}

fn dinic() {
    let (nodes, _edges, graph) = make_sample_graph();

    let mut dinic = Dinic::new(&graph);
    let objective_value = dinic.maximum_flow(nodes[0], nodes[5]).unwrap().objective_value;
    println!("maximum flow from {} to {} is {}", 0, 5, objective_value);

    let objective_value = dinic.maximum_flow(nodes[2], nodes[4]).unwrap().objective_value;
    println!("maximum flow from {} to {} is {}", 2, 4, objective_value);
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
    ford_fulkerson();
    dinic();
}
