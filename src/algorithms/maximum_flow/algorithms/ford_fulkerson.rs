use crate::{
    algorithms::maximum_flow::{
        algorithms::{macros::impl_maximum_flow_solver, solver::MaximumFlowSolver},
        edge::MaximumFlowEdge,
        residual_network::ResidualNetwork,
        result::MaxFlowResult,
        status::Status,
        validate::validate_input,
    },
    core::numeric::FlowNum,
    graph::{direction::Directed, graph::Graph, ids::NodeId},
};
use std::marker::PhantomData;

pub struct FordFulkerson<N, F> {
    rn: ResidualNetwork<N, F>,
    visited: Box<[bool]>,
    cutoff: Option<F>,
    phantom: PhantomData<N>,
}

impl<N, F> FordFulkerson<N, F>
where
    F: FlowNum,
{
    fn new(graph: &Graph<Directed, N, MaximumFlowEdge<F>>) -> Self {
        let rn = ResidualNetwork::new(graph);
        let num_nodes = rn.num_nodes;
        Self { rn, visited: vec![false; num_nodes].into_boxed_slice(), cutoff: None, phantom: PhantomData }
    }

    pub(crate) fn run(&mut self, source: NodeId, sink: NodeId) -> Result<MaxFlowResult<F>, Status> {
        validate_input(&self.rn, source, sink)?;
        // initialize
        self.rn.residual_capacities.copy_from_slice(&self.rn.upper);

        let mut residual = self.cutoff.unwrap_or_else(|| {
            self.rn
                .neighbors(source)
                .fold(F::zero(), |acc, arc_id| acc + self.rn.residual_capacities[arc_id.index()])
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

        Ok(MaxFlowResult { objective_value, flows: self.rn.get_flows(&self.rn.residual_capacities) })
    }

    fn dfs(&mut self, u: NodeId, sink: NodeId, flow: F) -> Option<F> {
        if u == sink {
            return Some(flow);
        }
        self.visited[u.index()] = true;

        for arc_id in self.rn.neighbors(u) {
            let to = self.rn.to[arc_id.index()];
            let residual_capacity = self.rn.residual_capacities[arc_id.index()];
            if self.visited[to.index()] || residual_capacity == F::zero() {
                continue;
            }

            if let Some(d) = self.dfs(to, sink, flow.min(residual_capacity)) {
                self.rn.push_flow(u, arc_id, d, false);
                return Some(d);
            }
        }
        None
    }

    pub fn cutoff(mut self, k: F) -> Self {
        self.cutoff = Some(k);
        self
    }

    pub fn clear_cutoff(&mut self) {
        self.cutoff = None;
    }
}

impl_maximum_flow_solver!(FordFulkerson, run);
