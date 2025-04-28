use crate::maximum_flow::csr::CSR;
use crate::maximum_flow::graph::Graph;
use crate::maximum_flow::status::Status;
use crate::maximum_flow::MaximumFlowSolver;
use num_traits::NumAssign;

#[derive(Default)]
pub struct ShortestAugmentingPath<Flow> {
    csr: CSR<Flow>,
    pub current_edge: Vec<usize>,
}

impl<Flow> MaximumFlowSolver<Flow> for ShortestAugmentingPath<Flow>
where
    Flow: NumAssign + Ord + Copy,
{
    fn solve(&mut self, graph: &mut Graph<Flow>, source: usize, sink: usize, upper: Option<Flow>) -> Result<Flow, Status> {
        if source >= graph.num_nodes() || sink >= graph.num_nodes() || source == sink {
            return Err(Status::BadInput);
        }

        self.csr.build(graph);
        self.csr.update_distances_to_sink(source, sink);
        self.current_edge.resize(self.csr.num_nodes, 0);

        let mut flow = Flow::zero();
        let mut residual = upper.unwrap_or_else(|| self.csr.neighbors(source).fold(Flow::zero(), |sum, i| sum + self.csr.upper[i]));
        while self.csr.distances_to_sink[source] < self.csr.num_nodes {
            self.current_edge.iter_mut().enumerate().for_each(|(u, e)| *e = self.csr.start[u]);
            if let Some(delta) = self.dfs(source, sink, residual) {
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
    Flow: NumAssign + Ord + Copy,
{
    pub fn solve(&mut self, graph: &mut Graph<Flow>, source: usize, sink: usize, upper: Option<Flow>) -> Result<Flow, Status> {
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

pub fn shortest_augmenting_path<Flow>(graph: &mut Graph<Flow>, source: usize, sink: usize, upper: Option<Flow>) -> Result<Flow, Status>
where
    Flow: NumAssign + Ord + Copy + Default,
{
    ShortestAugmentingPath::<Flow>::default().solve(graph, source, sink, upper)
}
