use crate::maximum_flow::csr::CSR;
use crate::maximum_flow::graph::Graph;
use crate::maximum_flow::status::Status;
use num_traits::NumAssign;

#[derive(Default)]
pub struct FordFulkerson<Flow> {
    csr: CSR<Flow>,
}

impl<Flow> FordFulkerson<Flow>
where
    Flow: NumAssign + Ord + Copy,
{
    pub fn solve(&mut self, source: usize, sink: usize, graph: &mut Graph<Flow>) -> Status {
        self.csr.build(graph);
        let mut visited = vec![false; self.csr.num_nodes];

        let upper = self.csr.neighbors(source).fold(Flow::zero(), |sum, i| sum + self.csr.upper[i]);
        let mut flow = Flow::zero();
        loop {
            visited.fill(false);
            match self.dfs(source, sink, upper, &mut visited) {
                Some(delta) => flow += delta,
                None => break,
            }
        }

        self.csr.set_flow(graph);
        Status::Optimal
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
                self.csr.push_flow(i, d);
                return Some(d);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_aoj() {
        let mut graph = Graph::<i32>::default();
        graph.add_nodes(4);
        graph.add_directed_edge(0, 1, 2);
        graph.add_directed_edge(0, 2, 1);
        graph.add_directed_edge(1, 2, 1);
        graph.add_directed_edge(1, 3, 1);
        graph.add_directed_edge(2, 3, 2);

        let status = FordFulkerson::default().solve(0, 3, &mut graph);
        assert_eq!(status, Status::Optimal);
        assert_eq!(graph.maximum_flow(0), 3);
    }
}
