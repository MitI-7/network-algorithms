use crate::algorithms::maximum_flow::csr::CSR;
use crate::core::graph::Graph;
use crate::algorithms::maximum_flow::status::Status;
use crate::algorithms::maximum_flow::FlowNum;
use crate::algorithms::maximum_flow::MaximumFlowSolver;
use crate::traits::One;
use core::ops::{Div, DivAssign, Mul, MulAssign};
use std::collections::VecDeque;
use crate::core::direction::Directed;
use crate::core::ids::NodeId;
use crate::edge::capacity::CapEdge;

#[derive(Default)]
pub struct CapacityScaling<Flow> {
    csr: CSR<Flow>,
    current_edge: Vec<usize>,
    que: VecDeque<usize>,
}

impl<Flow> MaximumFlowSolver<Flow> for CapacityScaling<Flow>
where
    Flow: FlowNum + One + Mul<Output = Flow> + MulAssign + Div<Output = Flow> + DivAssign,
{
    fn solve(&mut self, graph: &mut Graph<Directed, (), CapEdge<Flow>>, source: NodeId, sink: NodeId, upper: Option<Flow>) -> Result<Flow, Status> {
        if source.index() >= graph.num_nodes() || sink.index() >= graph.num_nodes() || source == sink {
            return Err(Status::BadInput);
        }

        self.csr.build(graph);
        self.current_edge.resize(self.csr.num_nodes, 0);
        let two = Flow::one() + Flow::one();

        let max_capacity = *self.csr.upper.iter().map(|f| f).max().unwrap_or(&Flow::zero());
        let mut delta = Flow::one();
        while delta <= max_capacity {
            delta *= two;
        }
        delta /= two;

        let mut residual = upper.unwrap_or_else(|| self.csr.neighbors(source.index()).fold(Flow::zero(), |sum, i| sum + self.csr.upper[i]));
        let mut flow = Flow::zero();
        while delta > Flow::zero() {
            // solve maximum flow in delta-residual network
            loop {
                self.bfs(source.index(), sink.index(), delta);

                // no s-t path
                if self.csr.distances_to_sink[source.index()] >= self.csr.num_nodes {
                    break;
                }

                self.current_edge.iter_mut().enumerate().for_each(|(u, e)| *e = self.csr.start[u]);
                match self.dfs(source.index(), sink.index(), residual, delta) {
                    Some(delta) => {
                        flow += delta;
                        residual -= delta;
                    }
                    None => break,
                }
            }
            delta /= two;
        }

        // copy
        self.csr.set_flow(graph);

        Ok(flow)
    }
}

impl<Flow> CapacityScaling<Flow>
where
    Flow: FlowNum + One + Mul<Output = Flow> + MulAssign + Div<Output = Flow> + DivAssign,
{
    pub fn solve(&mut self, graph: &mut Graph<Directed, (), CapEdge<Flow>>, source: NodeId, sink: NodeId, upper: Option<Flow>) -> Result<Flow, Status> {
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
