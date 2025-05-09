use crate::maximum_flow::csr::CSR;
use crate::core::graph::Graph;
use crate::maximum_flow::status::Status;
use crate::maximum_flow::FlowNum;
use crate::maximum_flow::MaximumFlowSolver;
use std::collections::VecDeque;
use crate::core::direction::Directed;
use crate::core::ids::NodeId;
use crate::edge::capacity::CapEdge;

#[derive(Default)]
pub struct EdmondsKarp<Flow> {
    csr: CSR<Flow>,
}

impl<Flow> MaximumFlowSolver<Flow> for EdmondsKarp<Flow>
where
    Flow: FlowNum,
{
    fn solve(&mut self, graph: &mut Graph<Directed, (), CapEdge<Flow>>, source: NodeId, sink: NodeId, upper: Option<Flow>) -> Result<Flow, Status> {
        if source.index() >= graph.num_nodes() || sink.index() >= graph.num_nodes() || source == sink {
            return Err(Status::BadInput);
        }

        self.csr.build(graph);
        let mut prev = vec![(usize::MAX, usize::MAX); self.csr.num_nodes];
        let mut visited = vec![false; self.csr.num_nodes];
        let mut residual = upper.unwrap_or_else(|| self.csr.neighbors(source.index()).fold(Flow::zero(), |acc, i| acc + self.csr.upper[i]));
        let mut flow = Flow::zero();
        while residual > Flow::zero() {
            prev.fill((usize::MAX, usize::MAX));
            visited.fill(false);

            // bfs
            let mut queue = VecDeque::from([source.index()]);
            while let Some(u) = queue.pop_front() {
                visited[u] = true;
                if u == sink.index() {
                    break;
                }

                for edge_id in self.csr.neighbors(u) {
                    let to = self.csr.to[edge_id];
                    if visited[to] || self.csr.residual_capacity(edge_id) == Flow::zero() {
                        continue;
                    }

                    queue.push_back(to);
                    prev[to] = (u, edge_id);
                }
            }

            if !visited[sink.index()] {
                break;
            }

            // calculate delta
            let mut delta = self.csr.residual_capacity(prev[sink.index()].1).min(residual);
            let mut v = sink.index();
            while v != source.index() {
                let (u, edge_id) = prev[v];
                delta = delta.min(self.csr.residual_capacity(edge_id));
                v = u;
            }
            assert!(delta > Flow::zero());

            // update flow
            let mut v = sink.index();
            while v != source.index() {
                let (u, edge_id) = prev[v];
                self.csr.push_flow(u, edge_id, delta, true);
                v = u;
            }
            flow += delta;
            residual -= delta;
        }

        self.csr.set_flow(graph);
        Ok(flow)
    }
}

impl<Flow> EdmondsKarp<Flow>
where
    Flow: FlowNum,
{
    pub fn solve(&mut self, graph: &mut Graph<Directed, (), CapEdge<Flow>>, source: NodeId, sink: NodeId, upper: Option<Flow>) -> Result<Flow, Status> {
        <Self as MaximumFlowSolver<Flow>>::solve(self, graph, source, sink, upper)
    }
}
