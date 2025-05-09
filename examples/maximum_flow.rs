// use network_algorithms::maximum_flow::{Dinic, Graph, PushRelabelHighestLabel};
// 
// fn make_sample_graph() -> Graph<i32> {
//     let mut graph = Graph::default();
//     graph.add_nodes(4);
// 
//     graph.add_directed_edge(0, 1, 2).unwrap();
//     graph.add_directed_edge(0, 2, 1).unwrap();
//     graph.add_directed_edge(1, 2, 1).unwrap();
//     graph.add_directed_edge(1, 3, 1).unwrap();
//     graph.add_directed_edge(2, 3, 2).unwrap();
// 
//     graph
// }
// 
// fn dinic_sample() {
//     let mut graph = make_sample_graph();
//     match Dinic::default().solve(&mut graph, 0, 3, None) {
//         Ok(value) => {
//             println!("maximum flow:{}", value);
//             for edge_id in 0..graph.num_edges() {
//                 println!("{:?}", graph.get_edge(edge_id).unwrap());
//             }
//         }
//         _ => unreachable!(),
//     }
// }
// 
// fn push_relabel() {
//     let mut graph = make_sample_graph();
//     match PushRelabelHighestLabel::default()
//         .set_value_only(true)
//         .set_global_relabel_freq(0.5)
//         .solve(&mut graph, 0, 3, None)
//     {
//         Ok(value) => {
//             println!("maximum flow:{}", value);
//             for edge_id in 0..graph.num_edges() {
//                 println!("{:?}", graph.get_edge(edge_id).unwrap());
//             }
//         }
//         _ => unreachable!(),
//     }
// }
// 
// fn main() {
//     println!("dinic");
//     dinic_sample();
// 
//     println!("push relabel");
//     push_relabel();
// }

fn main() {

}