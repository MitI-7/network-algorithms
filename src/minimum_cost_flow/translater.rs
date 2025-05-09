use crate::graph::minimum_cost_flow_graph::{Graph, Edge};
use crate::minimum_cost_flow::MinimumCostFlowNum;

pub fn translater<Flow>(graph: &Graph<Flow>) -> Graph<Flow> 
where Flow: MinimumCostFlowNum {
    let mut new_graph = graph.clone();
    new_graph.add_nodes(graph.num_nodes());
    for edge in new_graph.edges.iter_mut() {
        if edge.cost >= Flow::zero() {
            // new_graph.add_directed_edge(edge.from, edge.to, Flow::zero(), edge.upper - edge.lower, edge.cost);
            new_graph.b[edge.from] -= edge.lower;
            new_graph.b[edge.to] += edge.lower;
            edge.upper = edge.upper - edge.lower;
            edge.lower = Flow::zero();
            // new_graph.lowers.push(lower);
            // new_graph.is_reversed.push(false);
        } else {
            // new_graph.edges .push(Edge { from: edge.to, to: edge.from, flow: Flow::zero(), lower: Flow::zero(), upper: edge.upper - edge.lower, cost: -edge.cost });
            new_graph.b[edge.from] -= edge.upper;
            new_graph.b[edge.to] += edge.upper;
            edge.upper = edge.upper - edge.lower;
            edge.lower = Flow::zero();
            edge.cost = -edge.cost;
            (edge.from, edge.to) = (edge.to, edge.from);
            // self.lowers.push(lower);
            // self.is_reversed.push(true);
        }
    }
    new_graph
}