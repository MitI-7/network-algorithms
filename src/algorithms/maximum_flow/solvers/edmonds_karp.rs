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
        ids::{EdgeId, INVALID_ARC_ID, INVALID_NODE_ID, NodeId},
    },
};
use std::collections::VecDeque;

pub struct EdmondsKarp<F> {
    rn: ResidualNetwork<F>,
    cutoff: Option<F>,
    source: NodeId,
}

impl<F> EdmondsKarp<F>
where
    F: FlowNum,
{
    fn new<N>(graph: &Graph<Directed, N, MaximumFlowEdge<F>>) -> Self {
        let rn = ResidualNetwork::new(graph);
        Self { rn, cutoff: None, source: INVALID_NODE_ID }
    }

    fn run(&mut self, source: NodeId, sink: NodeId) -> Result<F, Status> {
        validate_input(&self.rn, source, sink)?;

        self.source = source;
        let mut prev = vec![(INVALID_NODE_ID, INVALID_ARC_ID); self.rn.num_nodes];
        let mut visited = vec![false; self.rn.num_nodes];
        let mut residual = self.cutoff.unwrap_or_else(|| {
            self.rn
                .neighbors(source)
                .fold(F::zero(), |acc, arc_id| acc + self.rn.upper[arc_id.index()])
        });
        let mut flow = F::zero();
        while residual > F::zero() {
            prev.fill((INVALID_NODE_ID, INVALID_ARC_ID));
            visited.fill(false);

            // bfs
            let mut queue = VecDeque::from([source]);
            while let Some(u) = queue.pop_front() {
                visited[u.index()] = true;
                if u == sink {
                    break;
                }

                for arc_id in self.rn.neighbors(u) {
                    let to = self.rn.to[arc_id.index()];
                    if visited[to.index()] || self.rn.residual_capacity(arc_id) == F::zero() {
                        continue;
                    }

                    queue.push_back(to);
                    prev[to.index()] = (u, arc_id);
                }
            }

            if !visited[sink.index()] {
                break;
            }

            // calculate delta
            let mut delta = self.rn.residual_capacity(prev[sink.index()].1).min(residual);
            let mut v = sink;
            while v != source {
                let (u, arc_id) = prev[v.index()];
                delta = delta.min(self.rn.residual_capacity(arc_id));
                v = u;
            }
            assert!(delta > F::zero());

            // update flow
            let mut v = sink;
            while v != source {
                let (u, arc_id) = prev[v.index()];
                self.rn.push_flow_without_excess(u, arc_id, delta);
                v = u;
            }
            flow += delta;
            residual -= delta;
        }

        Ok(flow)
    }
}

impl_maximum_flow_solver!(EdmondsKarp, run);
