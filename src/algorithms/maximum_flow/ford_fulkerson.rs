use crate::{
    algorithms::maximum_flow::{
        edge::MaximumFlowEdge, residual_network_core::ResidualNetworkCore, result::MaxFlowResult,
        solver::MaximumFlowSolver, status::Status, validate::validate_input,
    },
    core::numeric::FlowNum,
    graph::{direction::Directed, graph::Graph, ids::NodeId},
};
use std::marker::PhantomData;

#[derive(Default)]
pub struct FordFulkerson<N, F> {
    residual_capacities: Box<[F]>,
    excesses: Box<[F]>,
    visited: Box<[bool]>,
    phantom: PhantomData<N>,
}

impl<N, F> MaximumFlowSolver<N, F> for FordFulkerson<N, F>
where
    F: FlowNum,
{
    type Prepared = ResidualNetworkCore<N, F>;

    fn solve(
        &mut self,
        graph: &Graph<Directed, N, MaximumFlowEdge<F>>,
        source: NodeId,
        sink: NodeId,
        upper: Option<F>,
    ) -> Result<MaxFlowResult<F>, Status> {
        validate_input(graph, source, sink)?;
        let rn = ResidualNetworkCore::from_graph(graph);
        self.run_with_prepared(&rn, source, sink, upper)
    }

    fn prepare(
        &mut self,
        graph: &Graph<Directed, N, MaximumFlowEdge<F>>,
    ) -> Result<Self::Prepared, Status> {
        Ok(ResidualNetworkCore::from_graph(graph))
    }

    fn solve_with_prepared(
        &mut self,
        rn: &Self::Prepared,
        s: NodeId,
        t: NodeId,
        upper: Option<F>,
    ) -> Result<MaxFlowResult<F>, Status> {
        self.run_with_prepared(rn, s, t, upper)
    }
}

impl<N, F> FordFulkerson<N, F>
where
    F: FlowNum,
{
    fn run_with_prepared(
        &mut self,
        rn: &ResidualNetworkCore<N, F>,
        source: NodeId,
        sink: NodeId,
        upper: Option<F>,
    ) -> Result<MaxFlowResult<F>, Status> {
        // initialize
        if self.residual_capacities.len() != rn.num_edges * 2 {
            self.residual_capacities = vec![F::zero(); rn.num_edges * 2].into_boxed_slice();
        } else {
            self.residual_capacities.fill(F::zero());
        }
        if self.excesses.len() != rn.num_nodes {
            self.excesses = vec![F::zero(); rn.num_nodes].into_boxed_slice();
        } else {
            self.excesses.fill(F::zero());
        }

        self.residual_capacities.copy_from_slice(&rn.upper);

        self.visited = vec![false; rn.num_nodes].into_boxed_slice();

        let mut residual = upper.unwrap_or_else(|| {
            rn.neighbors(source).fold(F::zero(), |acc, arc_id| {
                acc + self.residual_capacities[arc_id.index()]
            })
        });

        let mut objective_value = F::zero();
        while residual > F::zero() {
            self.visited.fill(false);
            match self.dfs(rn, source, sink, residual) {
                Some(delta) => {
                    objective_value += delta;
                    residual -= delta;
                }
                None => break,
            }
        }

        Ok(MaxFlowResult {
            objective_value,
            flows: rn.get_flows(&self.residual_capacities),
        })
    }

    fn dfs(&mut self, rn: &ResidualNetworkCore<N, F>, u: NodeId, sink: NodeId, flow: F) -> Option<F> {
        if u == sink {
            return Some(flow);
        }
        self.visited[u.index()] = true;

        for arc_id in rn.neighbors(u) {
            let to = rn.to[arc_id.index()];
            let residual_capacity = self.residual_capacities[arc_id.index()];
            if self.visited[to.index()] || residual_capacity == F::zero() {
                continue;
            }

            if let Some(d) = self.dfs(rn, to, sink, flow.min(residual_capacity)) {
                rn.push_flow(u, arc_id, d, &mut self.residual_capacities, None);
                return Some(d);
            }
        }
        None
    }
}
