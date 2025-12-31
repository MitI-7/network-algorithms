use crate::{
    algorithms::maximum_flow::{
        edge::MaximumFlowEdge, residual_network::ResidualNetwork, solver::MaximumFlowSolver,
        status::Status,
    },
    core::numeric::FlowNum,
    graph::{direction::Directed, graph::Graph, ids::NodeId},
};

#[derive(Default)]
pub struct FordFulkerson<F> {
    csr: ResidualNetwork<F>,
}

impl<F> MaximumFlowSolver<F> for FordFulkerson<F>
where
    F: FlowNum,
{
    fn solve(
        &mut self,
        graph: &Graph<Directed, (), MaximumFlowEdge<F>>,
        source: NodeId,
        sink: NodeId,
        upper: Option<F>,
    ) -> Result<(F, Vec<F>), Status> {
        self.run(graph, source, sink, upper)
    }
}

impl<F> FordFulkerson<F>
where
    F: FlowNum,
{
    pub fn run(
        &mut self,
        graph: &Graph<Directed, (), MaximumFlowEdge<F>>,
        source: NodeId,
        sink: NodeId,
        upper: Option<F>,
    ) -> Result<(F, Vec<F>), Status> {
        if source.index() >= graph.num_nodes()
            || sink.index() >= graph.num_nodes()
            || source == sink
        {
            return Err(Status::BadInput);
        }

        self.csr.build(graph);
        let mut visited = vec![false; self.csr.num_nodes];

        let mut residual = upper.unwrap_or_else(|| {
            self.csr
                .neighbors(source.index())
                .fold(F::zero(), |acc, i| acc + self.csr.upper[i])
        });
        let mut flow = F::zero();
        while residual > F::zero() {
            visited.fill(false);
            match self.dfs(source.index(), sink.index(), residual, &mut visited) {
                Some(delta) => {
                    flow += delta;
                    residual -= delta;
                }
                None => break,
            }
        }

        let f = self.csr.set_flow(graph);

        Ok((flow, f))
    }

    fn dfs(&mut self, u: usize, sink: usize, flow: F, visited: &mut Vec<bool>) -> Option<F> {
        if u == sink {
            return Some(flow);
        }
        visited[u] = true;

        for i in self.csr.neighbors(u) {
            let to = self.csr.to[i];
            let residual_capacity = self.csr.residual_capacity(i);
            if visited[to] || residual_capacity == F::zero() {
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
