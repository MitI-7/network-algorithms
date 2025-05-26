use crate::algorithms::minimum_cut::minimum_cut::minimum_cut;
use crate::maximum_flow::{Dinic, MaximumFlowGraph};
use crate::minimum_cost_flow::MinimumCostFlowNum;
use crate::prelude::{CapEdge, Graph, NodeId, Undirected};

// Gomory-Hu tree construction
fn gomory_hu<F: MinimumCostFlowNum>(graph: &Graph<Undirected, (), CapEdge<F>>) -> Vec<(usize, usize, F)> {
    let mut parent = vec![0; graph.num_nodes()];
    for u in 1..graph.num_nodes() {
        parent[u] = 0;
    }

    let mut tree_edges = Vec::new();
    for u in 1..graph.num_nodes() {
        let mut tmp_graph = MaximumFlowGraph::default();
        let _nodes = tmp_graph.add_nodes(graph.num_nodes());
        for edge in graph.edges.iter() {
            tmp_graph.add_directed_edge(edge.u, edge.v, edge.data.upper);
            tmp_graph.add_directed_edge(edge.v, edge.u, edge.data.upper);
        }
        let p = parent[u];
        let mut dinic = Dinic::default();
        let (flow, _cut, reachable) = minimum_cut(&mut dinic, &mut tmp_graph, NodeId(u), NodeId(p)).unwrap();

        // update parent for other nodes
        for v in (u + 1)..graph.num_nodes() {
            if parent[v] == p && reachable[v] {
                parent[v] = u;
            }
        }
        // record tree edge
        tree_edges.push((u, p, flow));
    }

    tree_edges
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn main() {
        let mut graph = Graph::new_undirected();
        let nodes = graph.add_nodes(4);
        graph.add_edge(nodes[0], nodes[1], CapEdge { flow: 0, upper: 2 });
        graph.add_edge(nodes[0], nodes[2], CapEdge { flow: 0, upper: 1 });
        graph.add_edge(nodes[0], nodes[3], CapEdge { flow: 0, upper: 1 });
        graph.add_edge(nodes[1], nodes[2], CapEdge { flow: 0, upper: 1 });
        graph.add_edge(nodes[2], nodes[3], CapEdge { flow: 0, upper: 3 });

        let tree = gomory_hu(&graph);
        println!("Gomory-Hu Tree Edges (v, parent, cut_value):");
        for (u, v, w) in tree {
            println!("{} - {} : {}", u, v, w);
        }
    }
}
