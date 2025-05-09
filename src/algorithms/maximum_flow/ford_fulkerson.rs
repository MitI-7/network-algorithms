use crate::core::direction::Directed;
use crate::algorithms::maximum_flow::csr::CSR;
use crate::core::graph::Graph;
use crate::core::ids::NodeId;
use crate::edge::capacity::CapEdge;
use crate::algorithms::maximum_flow::status::Status;
use crate::algorithms::maximum_flow::FlowNum;
use crate::algorithms::maximum_flow::MaximumFlowSolver;

#[derive(Default)]
pub struct FordFulkerson<Flow> {
    csr: CSR<Flow>,
}

impl<Flow> MaximumFlowSolver<Flow> for FordFulkerson<Flow>
where
    Flow: FlowNum,
{
    fn solve(&mut self, graph: &mut Graph<Directed, (), CapEdge<Flow>>, source: NodeId, sink: NodeId, upper: Option<Flow>) -> Result<Flow, Status> {
        if source.index() >= graph.num_nodes() || sink.index() >= graph.num_nodes() || source == sink {
            return Err(Status::BadInput);
        }

        self.csr.build(graph);
        let mut visited = vec![false; self.csr.num_nodes];

        let mut residual = upper.unwrap_or_else(|| self.csr.neighbors(source.index()).fold(Flow::zero(), |acc, i| acc + self.csr.upper[i]));
        let mut flow = Flow::zero();
        while residual > Flow::zero() {
            visited.fill(false);
            match self.dfs(source.index(), sink.index(), residual, &mut visited) {
                Some(delta) => {
                    flow += delta;
                    residual -= delta;
                }
                None => break,
            }
        }

        self.csr.set_flow(graph);

        Ok(flow)
    }
}

impl<Flow> FordFulkerson<Flow>
where
    Flow: FlowNum,
{
    pub fn solve(&mut self, graph: &mut Graph<Directed, (), CapEdge<Flow>>, source: NodeId, sink: NodeId, upper: Option<Flow>) -> Result<Flow, Status> {
        <Self as MaximumFlowSolver<Flow>>::solve(self, graph, source, sink, upper)
    }

    fn dfs(&mut self, u: usize, sink: usize, flow: Flow, visited: &mut Vec<bool>) -> Option<Flow> {
        if u == sink {
            return Some(flow);
        }
        visited[u] = true;

        for i in self.csr.neighbors(u) {
            let to = self.csr.to[i];
            let residual_capacity = self.csr.residual_capacity(i);
            if visited[to] || residual_capacity == Flow::zero() {
                continue;
            }

            if let Some(d) = self.dfs(to, sink, flow.min(residual_capacity), visited) {
                self.csr.push_flow(u, i, d, true);
                return Some(d);
            }
        }
        None
    }
}
