use crate::{
    algorithms::maximum_flow::{
        edge::MaximumFlowEdge,
        residual_network::ResidualNetwork,
        solvers::{macros::impl_maximum_flow_solver, solver::MaximumFlowSolver},
        status::Status,
        validate::validate_input,
    },
    core::numeric::FlowNum,
    graph::{
        direction::Directed,
        graph::Graph,
        ids::{ArcId, EdgeId, INVALID_NODE_ID, NodeId},
    },
};
use crate::algorithms::maximum_flow::error::MaximumFlowError;

pub struct ShortestAugmentingPath<F> {
    rn: ResidualNetwork<F>,
    current_edge: Box<[usize]>,
    cutoff: Option<F>,
    source: Option<NodeId>,
}

impl<F> ShortestAugmentingPath<F>
where
    F: FlowNum,
{
    fn new<N>(graph: &Graph<Directed, N, MaximumFlowEdge<F>>) -> Self {
        let rn = ResidualNetwork::new(graph);
        let num_nodes = rn.num_nodes;

        Self { rn, current_edge: vec![0_usize; num_nodes].into_boxed_slice(), cutoff: None, source: None }
    }

    fn run(&mut self, source: NodeId, sink: NodeId) -> Result<F, MaximumFlowError> {
        validate_input(&self.rn, source, sink)?;

        self.source = Some(source);
        self.rn.update_distances_to_sink(source, sink);

        let mut flow = F::zero();
        let mut residual = self.cutoff.unwrap_or_else(|| {
            self.rn
                .neighbors(source)
                .fold(F::zero(), |sum, arc_id| sum + self.rn.upper[arc_id.index()])
        });
        while self.rn.distances_to_sink[source.index()] < self.rn.num_nodes {
            self.current_edge
                .iter_mut()
                .enumerate()
                .for_each(|(u, e)| *e = self.rn.start[u]);
            if let Some(delta) = self.dfs(source, sink, residual) {
                flow += delta;
                residual -= delta;
            }
        }

        Ok(flow)
    }

    fn dfs(&mut self, u: NodeId, sink: NodeId, upper: F) -> Option<F> {
        if u == sink {
            return Some(upper);
        }

        for arc_id in (self.current_edge[u.index()]..self.rn.start[u.index() + 1]).map(ArcId) {
            self.current_edge[u.index()] = arc_id.index();
            let to = self.rn.to[arc_id.index()];
            if self.rn.is_admissible_arc(u, arc_id) {
                // advance
                if let Some(delta) = self.dfs(to, sink, upper.min(self.rn.residual_capacity(arc_id))) {
                    self.rn.push_flow_without_excess(u, arc_id, delta);
                    return Some(delta);
                }
            }
        }

        // retreat
        self.rn.distances_to_sink[u.index()] = self.rn.num_nodes;
        for arc_id in self.rn.neighbors(u) {
            let to = self.rn.to[arc_id.index()];
            if self.rn.residual_capacity(arc_id) > F::zero() {
                self.rn.distances_to_sink[u.index()] =
                    self.rn.distances_to_sink[u.index()].min(self.rn.distances_to_sink[to.index()] + 1);
            }
        }

        None
    }
}

impl_maximum_flow_solver!(ShortestAugmentingPath, run);
