use network_algorithms::Graph;
use network_algorithms::algorithms::shortest_path::prelude::*;
use network_algorithms::direction::Directed;
use network_algorithms::ids::{EdgeId, NodeId};
use network_algorithms::prelude::maximum_flow::MaximumFlowGraph;

fn make_sample_graph() -> (Vec<NodeId>, Vec<EdgeId>, ShortestPathGraph<i32>) {
    let mut graph = ShortestPathGraph::default();
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

fn dijkstra() {
    let (nodes, edges, graph) = make_sample_graph();
    let mut solver = Dijkstra::new(&graph);
    match solver.solve(nodes[0]) {
        Ok(()) => {
            for &u in nodes.iter() {
                if solver.reached(u) {
                    println!("{}-{}({})", nodes[0].index(), u.index(), solver.distance(u).unwrap());
                } else {
                    println!("{}-{}(unreach)", nodes[0].index(), u.index());
                }
            }
        }
        _ => unreachable!(),
    }
}

fn dijkstra_original_edge() {
    struct MyEdge {
        distance: usize,
    }
    let mut graph: Graph<Directed, (), MyEdge> = Graph::new_directed();
    let nodes = graph.add_nodes(6);
    let mut edges = Vec::new();
    edges.push(graph.add_edge(nodes[0], nodes[1], MyEdge { distance: 3 }).unwrap());
    edges.push(graph.add_edge(nodes[0], nodes[2], MyEdge { distance: 3 }).unwrap());
    edges.push(graph.add_edge(nodes[1], nodes[2], MyEdge { distance: 2 }).unwrap());
    edges.push(graph.add_edge(nodes[1], nodes[3], MyEdge { distance: 3 }).unwrap());
    edges.push(graph.add_edge(nodes[2], nodes[4], MyEdge { distance: 2 }).unwrap());
    edges.push(graph.add_edge(nodes[3], nodes[4], MyEdge { distance: 4 }).unwrap());
    edges.push(graph.add_edge(nodes[3], nodes[5], MyEdge { distance: 2 }).unwrap());
    edges.push(graph.add_edge(nodes[4], nodes[5], MyEdge { distance: 3 }).unwrap());

    let mut solver = Dijkstra::new_graph_with(&graph, |_, e| e.data.distance);

    match solver.solve(nodes[0]) {
        Ok(()) => {
            for &u in nodes.iter() {
                if solver.reached(u) {
                    println!("{}-{}({})", nodes[0].index(), u.index(), solver.distance(u).unwrap());
                } else {
                    println!("{}-{}(unreach)", nodes[0].index(), u.index());
                }
            }
        }
        _ => unreachable!(),
    }
}

fn main() {
    println!("dijkstra");
    dijkstra();
    println!("sample");
    dijkstra_original_edge();
}
