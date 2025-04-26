use crate::maximum_flow::csr::CSR;
use crate::maximum_flow::graph::Graph;
use crate::maximum_flow::status::Status;
use crate::maximum_flow::MaximumFlowSolver;
use num_traits::NumAssign;
use std::collections::VecDeque;

#[derive(Default)]
pub struct CapacityScaling<Flow> {
    csr: CSR<Flow>,
    current_edge: Vec<usize>,
    que: VecDeque<usize>,
}

impl<Flow> MaximumFlowSolver<Flow> for CapacityScaling<Flow>
where
    Flow: NumAssign + Ord + Copy,
{
    fn solve(&mut self, graph: &mut Graph<Flow>, source: usize, sink: usize, upper: Option<Flow>) -> Result<Flow, Status> {
        if source >= graph.num_nodes() || sink >= graph.num_nodes() || source == sink {
            return Err(Status::BadInput);
        }

        self.csr.build(graph);
        self.current_edge.resize(self.csr.num_nodes, 0);
        let two = Flow::one() + Flow::one();

        let max_capacity = *self.csr.upper.iter().map(|f| f).max().unwrap();
        let mut delta = Flow::one();
        while delta <= max_capacity {
            delta *= two;
        }
        delta /= two;

        let upper = upper.unwrap_or_else(|| self.csr.neighbors(source).fold(Flow::zero(), |sum, i| sum + self.csr.upper[i]));
        let mut flow = Flow::zero();
        while delta > Flow::zero() {
            // solve maximum flow in delta-residual network
            loop {
                self.bfs(source, sink, delta);

                // no s-t path
                if self.csr.distances_to_sink[source] >= self.csr.num_nodes {
                    break;
                }

                self.current_edge.iter_mut().enumerate().for_each(|(u, e)| *e = self.csr.start[u]);
                match self.dfs(source, sink, upper, delta) {
                    Some(delta) => flow += delta,
                    None => break,
                }
            }
            delta /= two;
        }

        // copy
        for edge_id in 0..graph.num_edges() {
            let i = self.csr.edge_index_to_inside_edge_index[edge_id];
            graph.edges[edge_id].flow = self.csr.flow[i];
        }

        Ok(flow)
    }
}

impl<Flow> CapacityScaling<Flow>
where
    Flow: NumAssign + Ord + Copy,
{
    pub fn solve(&mut self, graph: &mut Graph<Flow>, source: usize, sink: usize, upper: Option<Flow>) -> Result<Flow, Status> {
        <Self as MaximumFlowSolver<Flow>>::solve(self, graph, source, sink, upper)
    }

    fn bfs(&mut self, source: usize, sink: usize, delta: Flow) {
        self.que.clear();
        self.que.push_back(sink);
        self.csr.distances_to_sink.fill(self.csr.num_nodes);
        self.csr.distances_to_sink[sink] = 0;

        while let Some(v) = self.que.pop_front() {
            for i in self.csr.neighbors(v) {
                // e.to -> v
                let to = self.csr.to[i];
                let rev = self.csr.rev[i];
                if self.csr.residual_capacity(rev) >= delta && self.csr.distances_to_sink[to] == self.csr.num_nodes {
                    self.csr.distances_to_sink[to] = self.csr.distances_to_sink[v] + 1;
                    if to != source {
                        self.que.push_back(to);
                    }
                }
            }
        }
    }

    fn dfs(&mut self, u: usize, sink: usize, upper: Flow, delta: Flow) -> Option<Flow> {
        if u == sink {
            return Some(upper);
        }

        let mut res = Flow::zero();
        for i in self.current_edge[u]..self.csr.start[u + 1] {
            self.current_edge[u] = i;
            let v = self.csr.to[i];
            let residual_capacity = self.csr.residual_capacity(i);

            if !self.csr.is_admissible_edge(u, i) || residual_capacity < delta {
                continue;
            }

            if let Some(d) = self.dfs(v, sink, residual_capacity.min(upper - res), delta) {
                self.csr.push_flow(i, d);
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
