use crate::{
    core::numeric::CostNum,
    ids::NodeId,
    algorithms::minimum_cost_flow::normalized_network::{NormalizedEdge, NormalizedNetwork},
};

// transforms the minimum cost flow problem into a problem with a single excess node and a single deficit node.
pub(crate) fn construct_extend_network_one_supply_one_demand<F>(
    graph: &NormalizedNetwork<'_, F>,
) -> (NodeId, NodeId, Vec<NormalizedEdge<F>>, Vec<F>)
where
    F: CostNum,
{
    let source = NodeId(graph.num_nodes());
    let sink = NodeId(source.index() + 1);
    let mut edges = Vec::new();
    let mut excesses = vec![F::zero(); graph.num_nodes() + 2];
    let total_excess_positive = graph
        .excesses()
        .iter()
        .filter(|&e| *e > F::zero())
        .fold(F::zero(), |sum, &e| sum + e);
    let total_excess_negative = graph
        .excesses()
        .iter()
        .filter(|&e| *e < F::zero())
        .fold(F::zero(), |sum, &e| sum + e);

    for u in 0..graph.num_nodes() {
        if u == source.index() || u == sink.index() {
            continue;
        }

        let excess = graph.excesses()[u];
        if excess > F::zero() {
            // source -> u
            edges.push(NormalizedEdge {
                u: source,
                v: NodeId(u),
                lower: F::zero(),
                upper: excess,
                cost: F::zero(),
                is_reversed: false,
            });
        } else if excess < F::zero() {
            // u -> sink
            edges.push(NormalizedEdge {
                u: NodeId(u),
                v: sink,
                lower: F::zero(),
                upper: -excess,
                cost: F::zero(),
                is_reversed: false,
            });
        }
        excesses[u] -= excess;
    }
    excesses[source.index()] = total_excess_positive;
    excesses[sink.index()] = total_excess_negative;

    (source, sink, edges, excesses)
}

pub(crate) fn construct_extend_network_feasible_solution<F>(
    graph: &NormalizedNetwork<F>,
) -> (NodeId, Vec<NormalizedEdge<F>>, Vec<F>, Vec<F>)
where
    F: CostNum,
{
    let inf_cost = graph
        .iter_edges()
        .map(|e| e.cost)
        .fold(F::one(), |acc, cost| acc + cost); // all edge costs are non-negative

    let root = NodeId(graph.num_nodes());
    let mut artificial_edges = Vec::new();
    let mut flows = vec![F::zero(); graph.num_edges()]; // flow in original graph is zero
    let mut fix_excess = vec![F::zero(); graph.num_nodes() + 1]; // original graph + root
    for u in 0..graph.num_nodes() {
        if u == root.index() {
            continue;
        }

        let excess = graph.excesses()[u];
        if excess >= F::zero() {
            // u -> root
            let edge = NormalizedEdge {
                u: NodeId(u),
                v: root,
                lower: F::zero(),
                upper: excess,
                cost: inf_cost,
                is_reversed: false,
            };
            flows.push(excess);
            artificial_edges.push(edge);
        } else {
            // root -> u
            let edge = NormalizedEdge {
                u: root,
                v: NodeId(u),
                lower: F::zero(),
                upper: -excess,
                cost: inf_cost,
                is_reversed: false,
            };
            flows.push(-excess);
            artificial_edges.push(edge);
        }
        fix_excess[u] -= excess;
        fix_excess[root.index()] += excess;
    }

    (root, artificial_edges, flows, fix_excess)
}
