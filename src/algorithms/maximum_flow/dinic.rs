use crate::algorithms::maximum_flow::csr::CSR;
use crate::core::graph::Graph;
use crate::core::direction::Directed;
use crate::core::ids::NodeId;
use crate::edge::capacity::CapEdge;
use crate::algorithms::maximum_flow::status::Status;
use crate::algorithms::maximum_flow::FlowNum;
use crate::algorithms::maximum_flow::MaximumFlowSolver;

#[derive(Default)]
pub struct Dinic<Flow> {
    pub csr: CSR<Flow>,
    current_edge: Vec<usize>,
}

impl<Flow> MaximumFlowSolver<Flow> for Dinic<Flow>
where
    Flow: FlowNum,
{
    fn solve(&mut self, graph: &mut Graph<Directed, (), CapEdge<Flow>>, source: NodeId, sink: NodeId, upper: Option<Flow>) -> Result<Flow, Status> {
        if source.index() >= graph.num_nodes() || sink.index() >= graph.num_nodes() || source == sink {
            return Err(Status::BadInput);
        }

        self.csr.build(graph);
        self.current_edge.resize(graph.num_nodes(), 0);

        let mut residual = upper.unwrap_or_else(|| self.csr.neighbors(source.index()).fold(Flow::zero(), |sum, i| sum + self.csr.upper[i]));
        let mut flow = Flow::zero();
        while residual > Flow::zero() {
            self.csr.update_distances_to_sink(source.index(), sink.index());

            // no s-t path
            if self.csr.distances_to_sink[source.index()] >= self.csr.num_nodes {
                break;
            }

            self.current_edge.iter_mut().enumerate().for_each(|(u, e)| *e = self.csr.start[u]);
            match self.dfs(source.index(), sink.index(), residual) {
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

impl<Flow> Dinic<Flow>
where
    Flow: FlowNum,
{
    pub fn solve(&mut self, graph: &mut Graph<Directed, (), CapEdge<Flow>>, source: NodeId, sink: NodeId, upper: Option<Flow>) -> Result<Flow, Status> {
        <Self as MaximumFlowSolver<Flow>>::solve(self, graph, source, sink, upper)
    }

    fn dfs(&mut self, u: usize, sink: usize, upper: Flow) -> Option<Flow> {
        if u == sink {
            return Some(upper);
        }

        let mut res = Flow::zero();
        for i in self.current_edge[u]..self.csr.start[u + 1] {
            self.current_edge[u] = i;

            let v = self.csr.to[i];
            let residual_capacity = self.csr.residual_capacity(i);

            if !self.csr.is_admissible_edge(u, i) {
                continue;
            }

            if let Some(d) = self.dfs(v, sink, residual_capacity.min(upper - res)) {
                self.csr.push_flow(u, i, d, true);
                res += d;
                if res == upper {
                    return Some(res);
                }
            }
        }
        self.current_edge[u] = self.csr.start[u + 1];
        self.csr.distances_to_sink[u] = self.csr.num_nodes;

        Some(res)
    }
}
