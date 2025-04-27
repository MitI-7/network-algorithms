use crate::minimum_cost_flow::csr::CSR;
use crate::minimum_cost_flow::graph::Graph;
use crate::minimum_cost_flow::status::Status;
use crate::minimum_cost_flow::MinimumCostFlowSolver;
use num_traits::NumAssign;
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::ops::Neg;

#[derive(Default)]
pub struct SuccessiveShortestPath<Flow> {
    csr: CSR<Flow>,
}

impl<Flow> MinimumCostFlowSolver<Flow> for SuccessiveShortestPath<Flow>
where
    Flow: NumAssign + Neg<Output = Flow> + Ord + Copy,
{
    fn solve(&mut self, graph: &mut Graph<Flow>) -> Result<Flow, Status> {
        if graph.is_unbalance() {
            return Err(Status::Unbalanced);
        }
        self.csr.build(graph);

        for s in 0..self.csr.num_nodes {
            while self.csr.excesses[s] > Flow::zero() {
                match self.calculate_distance(s) {
                    Some((t, visited, dist, prev)) => {
                        // update potentials
                        for u in 0..self.csr.num_nodes {
                            if visited[u] {
                                self.csr.potentials[u] = self.csr.potentials[u] - dist[u].unwrap() + dist[t].unwrap();
                            }
                        }
                        // update flow
                        self.update_flow(s, t, prev);
                    }
                    None => break,
                }
            }
        }

        self.csr.set_flow(graph);

        if self.csr.excesses.iter().all(|&e| e == Flow::zero()) {
            Ok(graph.minimum_cost())
        } else {
            Err(Status::Infeasible)
        }
    }
}

impl<Flow> SuccessiveShortestPath<Flow>
where
    Flow: NumAssign + Neg<Output = Flow> + Ord + Copy,
{
    pub fn solve(&mut self, graph: &mut Graph<Flow>) -> Result<Flow, Status> {
        <Self as MinimumCostFlowSolver<Flow>>::solve(self, graph)
    }

    pub fn calculate_distance(&mut self, s: usize) -> Option<(usize, Vec<bool>, Vec<Option<Flow>>, Vec<Option<usize>>)> {
        let mut prev = vec![None; self.csr.num_nodes];
        let mut bh = BinaryHeap::new();
        let mut dist: Vec<Option<Flow>> = vec![None; self.csr.num_nodes];
        let mut visited = vec![false; self.csr.num_nodes];

        bh.push((Reverse(Flow::zero()), s));
        dist[s] = Some(Flow::zero());

        while let Some((d, u)) = bh.pop() {
            if visited[u] {
                continue;
            }
            visited[u] = true;

            if self.csr.excesses[u] < Flow::zero() {
                return Some((u, visited, dist, prev));
            }

            for edge_id in self.csr.start[u]..self.csr.start[u + 1] {
                if self.csr.residual_capacity(edge_id) == Flow::zero() {
                    continue;
                }

                let to = self.csr.to[edge_id];
                let new_dist = d.0 + self.csr.reduced_cost(u, edge_id);
                if dist[to].is_none() || dist[to].unwrap() > new_dist {
                    dist[to] = Some(new_dist);
                    prev[to] = Some(edge_id);
                    bh.push((Reverse(new_dist), to));
                }
            }
        }

        None
    }

    fn update_flow(&mut self, s: usize, t: usize, prev: Vec<Option<usize>>) {
        debug_assert!(self.csr.excesses[s] > Flow::zero() && self.csr.excesses[t] < Flow::zero());

        // calculate delta
        let mut delta = self.csr.excesses[s].min(-self.csr.excesses[t]);
        {
            let mut v = t;
            while let Some(edge_idx) = prev[v] {
                delta = delta.min(self.csr.residual_capacity(edge_idx));
                let rev = self.csr.rev[edge_idx];
                v = self.csr.to[rev];
            }
            delta = delta.min(self.csr.excesses[v]);
            debug_assert_eq!(s, v);
            debug_assert!(delta > Flow::zero());
        }

        // update flow
        {
            let mut v = t;
            while let Some(edge_idx) = prev[v] {
                // push
                let rev = self.csr.rev[edge_idx];
                self.csr.flow[edge_idx] += delta;
                self.csr.flow[rev] -= delta;
                v = self.csr.to[rev];
            }
            debug_assert_eq!(s, v);
        }

        self.csr.excesses[t] += delta;
        self.csr.excesses[s] -= delta;
    }
}
