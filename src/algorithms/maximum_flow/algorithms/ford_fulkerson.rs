use crate::{
    algorithms::maximum_flow::{
        algorithms::solver::{BuildMaximumFlowSolver, MaximumFlowSolver},
        edge::MaximumFlowEdge,
        residual_network_core::ResidualNetworkCore,
        result::MaxFlowResult,
        status::Status,
        validate::validate_input,
    },
    core::numeric::FlowNum,
    graph::{direction::Directed, graph::Graph, ids::NodeId},
};
use std::marker::PhantomData;

#[derive(Default)]
pub struct FordFulkerson<N, F> {
    rn: ResidualNetworkCore<N, F>,
    residual_capacities: Box<[F]>,
    visited: Box<[bool]>,
    phantom: PhantomData<N>,
}

impl<N, F> BuildMaximumFlowSolver<N, F> for FordFulkerson<N, F>
where
    F: FlowNum,
{
    fn new(graph: &Graph<Directed, N, MaximumFlowEdge<F>>) -> Self {
        FordFulkerson::new(graph)
    }
}

impl<N, F> MaximumFlowSolver<N, F> for FordFulkerson<N, F>
where
    F: FlowNum,
{
    fn solve(&mut self, source: NodeId, sink: NodeId, upper: Option<F>) -> Result<MaxFlowResult<F>, Status> {
        FordFulkerson::run(self, source, sink, upper)
    }
}

impl<N, F> FordFulkerson<N, F>
where
    F: FlowNum,
{
    fn new(graph: &Graph<Directed, N, MaximumFlowEdge<F>>) -> Self {
        let rn = ResidualNetworkCore::from_graph(graph);
        let num_nodes = rn.num_nodes;
        let num_edges = rn.num_edges;
        Self {
            rn,
            residual_capacities: vec![F::zero(); num_edges * 2].into_boxed_slice(),
            visited: vec![false; num_nodes].into_boxed_slice(),
            phantom: PhantomData,
        }
    }

    fn run(&mut self, source: NodeId, sink: NodeId, upper: Option<F>) -> Result<MaxFlowResult<F>, Status> {
        validate_input(&self.rn, source, sink)?;
        
        self.residual_capacities.copy_from_slice(&self.rn.upper);

        let mut residual = upper.unwrap_or_else(|| {
            self.rn
                .neighbors(source)
                .fold(F::zero(), |acc, arc_id| acc + self.residual_capacities[arc_id.index()])
        });

        let mut objective_value = F::zero();
        while residual > F::zero() {
            self.visited.fill(false);
            match self.dfs(source, sink, residual) {
                Some(delta) => {
                    objective_value += delta;
                    residual -= delta;
                }
                None => break,
            }
        }

        Ok(MaxFlowResult { objective_value, flows: self.rn.get_flows(&self.residual_capacities) })
    }

    fn dfs(&mut self, u: NodeId, sink: NodeId, flow: F) -> Option<F> {
        if u == sink {
            return Some(flow);
        }
        self.visited[u.index()] = true;

        for arc_id in self.rn.neighbors(u) {
            let to = self.rn.to[arc_id.index()];
            let residual_capacity = self.residual_capacities[arc_id.index()];
            if self.visited[to.index()] || residual_capacity == F::zero() {
                continue;
            }

            if let Some(d) = self.dfs(to, sink, flow.min(residual_capacity)) {
                self.rn.push_flow(u, arc_id, d, &mut self.residual_capacities, None);
                return Some(d);
            }
        }
        None
    }
}
