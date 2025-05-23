use crate::core::direction::Directed;
use crate::core::graph::Graph;
use crate::core::ids::NodeId;
use crate::edge::capacity::CapEdge;
use crate::algorithms::maximum_flow::csr::CSR;
use crate::algorithms::maximum_flow::status::Status;
use crate::algorithms::maximum_flow::FlowNum;
use crate::algorithms::maximum_flow::MaximumFlowSolver;

#[derive(Default)]
pub struct ShortestAugmentingPath<Flow> {
    csr: CSR<Flow>,
    pub current_edge: Vec<usize>,
}

impl<Flow> MaximumFlowSolver<Flow> for ShortestAugmentingPath<Flow>
where
    Flow: FlowNum,
{
    fn solve(&mut self, graph: &mut Graph<Directed, (), CapEdge<Flow>>, source: NodeId, sink: NodeId, upper: Option<Flow>) -> Result<Flow, Status> {
        if source.index() >= graph.num_nodes() || sink.index() >= graph.num_nodes() || source == sink {
            return Err(Status::BadInput);
        }

        self.csr.build(graph);
        self.csr.update_distances_to_sink(source.index(), sink.index());
        self.current_edge.resize(self.csr.num_nodes, 0);

        let mut flow = Flow::zero();
        let mut residual = upper.unwrap_or_else(|| self.csr.neighbors(source.index()).fold(Flow::zero(), |sum, i| sum + self.csr.upper[i]));
        while self.csr.distances_to_sink[source.index()] < self.csr.num_nodes {
            self.current_edge.iter_mut().enumerate().for_each(|(u, e)| *e = self.csr.start[u]);
            if let Some(delta) = self.dfs(source.index(), sink.index(), residual) {
                flow += delta;
                residual -= delta;
            }
        }

        self.csr.set_flow(graph);
        Ok(flow)
    }
}

impl<Flow> ShortestAugmentingPath<Flow>
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

        for i in self.current_edge[u]..self.csr.start[u + 1] {
            self.current_edge[u] = i;
            let to = self.csr.to[i];
            if self.csr.is_admissible_edge(u, i) {
                // advance
                if let Some(delta) = self.dfs(to, sink, upper.min(self.csr.residual_capacity(i))) {
                    self.csr.push_flow(u, i, delta, true);
                    return Some(delta);
                }
            }
        }

        // retreat
        self.csr.distances_to_sink[u] = self.csr.num_nodes;
        for i in self.csr.neighbors(u) {
            let to = self.csr.to[i];
            if self.csr.residual_capacity(i) > Flow::zero() {
                self.csr.distances_to_sink[u] = self.csr.distances_to_sink[u].min(self.csr.distances_to_sink[to] + 1);
            }
        }

        None
    }
}
