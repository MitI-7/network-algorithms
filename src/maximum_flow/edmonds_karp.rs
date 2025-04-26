use crate::maximum_flow::csr::CSR;
use crate::maximum_flow::graph::Graph;
use crate::maximum_flow::status::Status;
use crate::maximum_flow::MaximumFlowSolver;
use num_traits::NumAssign;
use std::collections::VecDeque;

#[derive(Default)]
pub struct EdmondsKarp<Flow> {
    csr: CSR<Flow>,
}

impl<Flow> MaximumFlowSolver<Flow> for EdmondsKarp<Flow>
where
    Flow: NumAssign + Ord + Copy,
{
    fn solve(&mut self, graph: &mut Graph<Flow>, source: usize, sink: usize, upper: Option<Flow>) -> Result<Flow, Status> {
        if source >= graph.num_nodes() || sink >= graph.num_nodes() || source == sink {
            return Err(Status::BadInput);
        }

        self.csr.build(graph);
        let mut prev = vec![(usize::MAX, usize::MAX); self.csr.num_nodes];
        let mut visited = vec![false; self.csr.num_nodes];
        let upper = upper.unwrap_or_else(|| self.csr.neighbors(source).fold(Flow::zero(), |sum, i| sum + self.csr.upper[i]));
        let mut f = Flow::zero();
        loop {
            prev.fill((usize::MAX, usize::MAX));
            visited.fill(false);

            // bfs
            let mut queue = VecDeque::from([source]);
            while let Some(u) = queue.pop_front() {
                visited[u] = true;
                if u == sink {
                    break;
                }

                for edge_id in self.csr.start[u]..self.csr.start[u + 1] {
                    let to = self.csr.to[edge_id];
                    if visited[to] || self.csr.residual_capacity(edge_id) == Flow::zero() {
                        continue;
                    }

                    queue.push_back(to);
                    prev[to] = (u, edge_id);
                }
            }

            if !visited[sink] {
                break;
            }

            // calculate delta
            let mut delta = self.csr.residual_capacity(prev[sink].1);
            let mut v = sink;
            while v != source {
                let (u, edge_id) = prev[v];
                delta = delta.min(self.csr.residual_capacity(edge_id));
                v = u;
            }

            assert!(delta > Flow::zero());

            // update flow
            let mut v = sink;
            while v != source {
                let (u, edge_id) = prev[v];
                self.csr.push_flow(edge_id, delta);
                v = u;
            }
            f += delta;
        }

        self.csr.set_flow(graph);
        Ok(f)
    }
}

impl<Flow> EdmondsKarp<Flow>
where
    Flow: NumAssign + Ord + Copy + std::fmt::Display,
{
    pub fn solve(&mut self, graph: &mut Graph<Flow>, source: usize, sink: usize, upper: Option<Flow>) -> Result<Flow, Status> {
        <Self as MaximumFlowSolver<Flow>>::solve(self, graph, source, sink, upper)
    }
}
