use crate::minimum_cost_flow::csr::CSR;
use crate::minimum_cost_flow::graph::Graph;
use crate::minimum_cost_flow::status::Status;
use crate::minimum_cost_flow::MinimumCostFlowSolver;
use num_traits::NumAssign;
use std::ops::Neg;

#[derive(Default)]
pub struct CycleCanceling<Flow> {
    csr: CSR<Flow>,
}
impl<Flow> MinimumCostFlowSolver<Flow> for CycleCanceling<Flow>
where
    Flow: NumAssign + Neg<Output = Flow> + Ord + Copy,
{
    fn solve(&mut self, graph: &mut Graph<Flow>) -> Result<Flow, Status> {
        let (_source, artificial_nodes, artificial_edges) = graph.construct_extend_network_feasible_solution();
        self.csr.build(graph);

        let mut prev = vec![(usize::MAX, usize::MAX); self.csr.num_nodes];
        while let Some(start) = self.find_negative_cycle(&mut prev) {
            let (mut v, idx) = prev[start];
            let mut delta = self.csr.residual_capacity(idx);
            let mut cycle = vec![idx];
            while v != start {
                let (u, idx) = prev[v];
                cycle.push(idx);
                delta = delta.min(self.csr.residual_capacity(idx));
                v = u;
            }
            assert!(delta > Flow::zero());

            for idx in cycle {
                let rev = self.csr.rev[idx];
                self.csr.flow[idx] += delta;
                self.csr.flow[rev] -= delta;
            }
        }

        self.csr.set_flow(graph);

        let status = if artificial_edges.iter().all(|&edge_id| graph.edges[edge_id].flow == Flow::zero()) {
            Status::Optimal
        } else {
            Status::Infeasible
        };
        graph.remove_artificial_sub_graph(&artificial_nodes, &artificial_edges);

        if status == Status::Optimal {
            Ok(graph.minimum_cost())
        } else {
            Err(status)
        }
    }
}

impl<Flow> CycleCanceling<Flow>
where
    Flow: NumAssign + Neg<Output = Flow> + Ord + Copy,
{
    pub fn solve(&mut self, graph: &mut Graph<Flow>) -> Result<Flow, Status> {
        <Self as MinimumCostFlowSolver<Flow>>::solve(self, graph)
    }

    fn find_negative_cycle(&self, prev: &mut [(usize, usize)]) -> Option<usize> {
        let mut start = usize::MAX;
        let mut dist = vec![Flow::zero(); self.csr.num_nodes];
        for _ in 0..self.csr.num_nodes {
            let mut updated = false;
            for u in 0..self.csr.num_nodes {
                for edge_index in self.csr.neighbors(u) {
                    let to = self.csr.to[edge_index];
                    let cost = self.csr.cost[edge_index];
                    if self.csr.residual_capacity(edge_index) > Flow::zero() && dist[u] + cost < dist[to] {
                        dist[to] = dist[u] + cost;
                        prev[to] = (u, edge_index);
                        start = u;
                        updated = true;
                    }
                }
            }
            if !updated {
                return None;
            }
        }

        let mut v = start;
        let mut visited = vec![false; self.csr.num_nodes];
        loop {
            let (u, _) = prev[v];
            if visited[u] {
                return Some(v);
            }
            visited[u] = true;
            v = u;
        }
    }
}
