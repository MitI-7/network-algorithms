use crate::{
    algorithms::maximum_flow::{
        edge::MaximumFlowEdge,
        error::MaximumFlowError,
        residual_network::ResidualNetwork,
        solvers::{macros::impl_maximum_flow_solver, solver::MaximumFlowSolver},
        status::Status,
        validate::validate_input,
    },
    core::numeric::FlowNum,
    graph::{
        direction::Directed,
        graph::Graph,
        ids::{EdgeId, NodeId},
    },
};

pub struct FordFulkerson<F> {
    status: Status,
    source: Option<NodeId>,

    rn: ResidualNetwork<F>,
    visited: Box<[bool]>,
    cutoff: Option<F>,
}

impl<F> FordFulkerson<F>
where
    F: FlowNum,
{
    fn new<N>(graph: &Graph<Directed, N, MaximumFlowEdge<F>>) -> Self {
        let rn = ResidualNetwork::new(graph);
        let num_nodes = rn.num_nodes;
        Self {
            status: Status::NotSolved,
            source: None,
            rn,
            visited: vec![false; num_nodes].into_boxed_slice(),
            cutoff: None,
        }
    }

    pub(crate) fn run(&mut self, source: NodeId, sink: NodeId) -> Result<F, MaximumFlowError> {
        validate_input(&self.rn, source, sink)?;

        // initialize
        self.source = Some(source);
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

        self.status = Status::Optimal;
        Ok(objective_value)
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
                self.rn.push_flow_without_excess(u, arc_id, d);
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
