use std::marker::PhantomData;
use crate::{
    algorithms::maximum_flow::{
        edge::MaximumFlowEdge, residual_network::ResidualNetwork, result::MaxFlowResult,
        solver::MaximumFlowSolver, status::Status,
    },
    core::numeric::FlowNum,
    graph::{direction::Directed, graph::Graph, ids::NodeId},
};

#[derive(Default)]
pub struct FordFulkerson<N, F> {
    rn: ResidualNetwork<N, F>,
    phantom: PhantomData<N>,
}

impl<N, F> MaximumFlowSolver<N, F> for FordFulkerson<N, F>
where
    F: FlowNum,
{
    fn solve(
        &mut self,
        graph: &Graph<Directed, N, MaximumFlowEdge<F>>,
        source: NodeId,
        sink: NodeId,
        upper: Option<F>,
    ) -> Result<MaxFlowResult<F>, Status> {
        self.run(graph, source, sink, upper)
    }

    fn minimum_cut(
        &mut self,
        _graph: &Graph<Directed, N, MaximumFlowEdge<F>>,
        _source: NodeId,
        _sink: NodeId,
        _upper: Option<F>,
    ) -> Result<MaxFlowResult<F>, Status> {
        todo!()
    }
}

impl<N, F> FordFulkerson<N, F>
where
    F: FlowNum,
{
    fn run(
        &mut self,
        graph: &Graph<Directed, N, MaximumFlowEdge<F>>,
        source: NodeId,
        sink: NodeId,
        upper: Option<F>,
    ) -> Result<MaxFlowResult<F>, Status> {
        if source.index() >= graph.num_nodes()
            || sink.index() >= graph.num_nodes()
            || source == sink
        {
            return Err(Status::BadInput);
        }

        self.rn.build(graph);
        let mut visited = vec![false; self.rn.num_nodes];

        let mut residual = upper.unwrap_or_else(|| {
            self.rn
                .neighbors(source.index())
                .fold(F::zero(), |acc, i| acc + self.rn.upper[i])
        });
        let mut objective_value = F::zero();
        while residual > F::zero() {
            visited.fill(false);
            match self.dfs(source.index(), sink.index(), residual, &mut visited) {
                Some(delta) => {
                    objective_value += delta;
                    residual -= delta;
                }
                None => break,
            }
        }

        Ok(MaxFlowResult {
            objective_value,
            flows: self.rn.get_flows(),
        })
    }

    fn dfs(&mut self, u: usize, sink: usize, flow: F, visited: &mut Vec<bool>) -> Option<F> {
        if u == sink {
            return Some(flow);
        }
        visited[u] = true;

        for i in self.rn.neighbors(u) {
            let to = self.rn.to[i];
            let residual_capacity = self.rn.residual_capacity(i);
            if visited[to] || residual_capacity == F::zero() {
                continue;
            }

            if let Some(d) = self.dfs(to, sink, flow.min(residual_capacity), visited) {
                self.rn.push_flow(u, i, d, true);
                return Some(d);
            }
        }
        None
    }
}
