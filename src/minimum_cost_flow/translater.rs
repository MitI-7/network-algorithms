use crate::core::graph::Graph;
use crate::core::direction::Directed;
use crate::edge::capacity_cost::CapCostEdge;
use crate::node::excess::ExcessNode;
use crate::minimum_cost_flow::MinimumCostFlowNum;

pub fn translater<Flow>(graph: &Graph<Directed, ExcessNode<Flow>, CapCostEdge<Flow>>) -> Graph<Directed, ExcessNode<Flow>, CapCostEdge<Flow>>
where Flow: MinimumCostFlowNum {
    let mut new_graph = graph.clone();
    new_graph.add_nodes(graph.num_nodes());
    for edge in new_graph.edges.iter_mut() {
        if edge.data.cost >= Flow::zero() {
            // new_graph.add_directed_edge(edge.from, edge.to, Flow::zero(), edge.upper - edge.lower, edge.cost);
            new_graph.nodes[edge.from.index()].b -= edge.data.lower;
            new_graph.nodes[edge.to.index()].b += edge.data.lower;
            edge.data.upper = edge.data.upper - edge.data.lower;
            edge.data.lower = Flow::zero();
            // new_graph.lowers.push(lower);
            // new_graph.is_reversed.push(false);
        } else {
            // new_graph.edges .push(Edge { from: edge.to, to: edge.from, flow: Flow::zero(), lower: Flow::zero(), upper: edge.upper - edge.lower, cost: -edge.cost });
            new_graph.nodes[edge.from.index()].b -= edge.data.upper;
            new_graph.nodes[edge.to.index()].b += edge.data.upper;
            edge.data.upper = edge.data.upper - edge.data.lower;
            edge.data.lower = Flow::zero();
            edge.data.cost = -edge.data.cost;
            (edge.from, edge.to) = (edge.to, edge.from);
            // self.lowers.push(lower);
            // self.is_reversed.push(true);
        }
    }
    new_graph
}