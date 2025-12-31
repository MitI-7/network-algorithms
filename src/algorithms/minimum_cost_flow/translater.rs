use crate::algorithms::minimum_cost_flow::{
    MinimumCostFlowNum, edge::MinimumCostFlowEdge, node::MinimumCostFlowNode,
};
use crate::graph::{direction::Directed, graph::Graph};

pub fn translater<F>(
    graph: &Graph<Directed, MinimumCostFlowNode<F>, MinimumCostFlowEdge<F>>,
) -> Graph<Directed, MinimumCostFlowNode<F>, MinimumCostFlowEdge<F>>
where
    F: MinimumCostFlowNum,
{
    let mut new_graph = graph.clone();
    // new_graph.add_nodes(graph.num_nodes());
    for edge in new_graph.edges.iter_mut() {
        if edge.data.cost >= F::zero() {
            // new_graph.add_directed_edge(edge.from, edge.to, Flow::zero(), edge.upper - edge.lower, edge.cost);
            new_graph.nodes[edge.u.index()].data.b -= edge.data.lower;
            new_graph.nodes[edge.v.index()].data.b += edge.data.lower;
            edge.data.upper = edge.data.upper - edge.data.lower;
            edge.data.lower = F::zero();
            // new_graph.lowers.push(lower);
            // new_graph.is_reversed.push(false);
        } else {
            // new_graph.edges .push(Edge { from: edge.to, to: edge.from, flow: Flow::zero(), lower: Flow::zero(), upper: edge.upper - edge.lower, cost: -edge.cost });
            new_graph.nodes[edge.u.index()].data.b -= edge.data.upper;
            new_graph.nodes[edge.v.index()].data.b += edge.data.upper;
            edge.data.upper = edge.data.upper - edge.data.lower;
            edge.data.lower = F::zero();
            edge.data.cost = -edge.data.cost;
            (edge.u, edge.v) = (edge.v, edge.u);
            // self.lowers.push(lower);
            // self.is_reversed.push(true);
        }
    }
    new_graph
}
