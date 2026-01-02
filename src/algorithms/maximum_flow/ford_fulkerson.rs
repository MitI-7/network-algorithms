use crate::{
    algorithms::maximum_flow::{
        edge::MaximumFlowEdge, residual_network::ResidualNetwork, result::MaxFlowResult,
        solver::MaximumFlowSolver, status::Status,
    },
    core::numeric::FlowNum,
    graph::{direction::Directed, graph::Graph, ids::NodeId},
};
use std::marker::PhantomData;
use crate::maximum_flow::validate::validate_input;

#[derive(Default)]
pub struct FordFulkerson<N, F> {
    rn: ResidualNetwork<N, F>,
    phantom: PhantomData<N>,
}

impl<N, F> MaximumFlowSolver<N, F> for FordFulkerson<N, F>
where
    F: FlowNum,
{
    fn solve(
        &mut self,
        graph: &Graph<Directed, N, MaximumFlowEdge<F>>,
        source: NodeId,
        sink: NodeId,
        upper: Option<F>,
    ) -> Result<MaxFlowResult<F>, Status> {
        self.run(graph, source, sink, upper)
    }
}

impl<N, F> FordFulkerson<N, F>
where
    F: FlowNum,
{
    fn run(
        &mut self,
        graph: &Graph<Directed, N, MaximumFlowEdge<F>>,
        source: NodeId,
        sink: NodeId,
        upper: Option<F>,
    ) -> Result<MaxFlowResult<F>, Status> {
        validate_input(graph, source, sink)?;

        self.rn.build(graph);
        let mut visited = vec![false; self.rn.num_nodes];

        let mut residual = upper.unwrap_or_else(|| {
            self.rn
                .neighbors(source)
                .fold(F::zero(), |acc, arc_id| acc + self.rn.upper[arc_id.index()])
        });
        let mut objective_value = F::zero();
        while residual > F::zero() {
            visited.fill(false);
            match self.dfs(source, sink, residual, &mut visited) {
                Some(delta) => {
                    objective_value += delta;
                    residual -= delta;
                }
                None => break,
            }
        }

        Ok(MaxFlowResult {
            objective_value,
            flows: self.rn.get_flows(),
        })
    }

    fn dfs(&mut self, u: NodeId, sink: NodeId, flow: F, visited: &mut Vec<bool>) -> Option<F> {
        if u == sink {
            return Some(flow);
        }
        visited[u.index()] = true;

        for arc_id in self.rn.neighbors(u) {
            let to = self.rn.to[arc_id.index()];
            let residual_capacity = self.rn.residual_capacity(arc_id);
            if visited[to.index()] || residual_capacity == F::zero() {
                continue;
            }

            if let Some(d) = self.dfs(to, sink, flow.min(residual_capacity), visited) {
                self.rn.push_flow(u, arc_id, d, true);
                return Some(d);
            }
        }
        None
    }
}
