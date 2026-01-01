use crate::algorithms::minimum_cost_flow::edge::MinimumCostFlowEdge;
use crate::algorithms::minimum_cost_flow::node::MinimumCostFlowNode;
use crate::algorithms::minimum_cost_flow::{
    MinimumCostFlowNum, residual_network::ResidualNetwork, solver::MinimumCostFlowSolver, status::Status,
    translater::translater,
};
use crate::graph::{direction::Directed, graph::Graph, ids::EdgeId};
use std::{cmp::Reverse, collections::BinaryHeap};
use crate::algorithms::minimum_cost_flow::normalized_network::NormalizedNetwork;
// use crate::graph::node::ExcessNode;

#[derive(Default)]
pub struct SuccessiveShortestPath<F> {
    csr: ResidualNetwork<F>,
}

impl<F> MinimumCostFlowSolver<F> for SuccessiveShortestPath<F>
where
    F: MinimumCostFlowNum,
{
    fn solve(&mut self, graph: &mut Graph<Directed, MinimumCostFlowNode<F>, MinimumCostFlowEdge<F>>) -> Result<F, Status> {
        if (0..graph.num_nodes())
            .into_iter()
            .fold(F::zero(), |sum, u| sum + graph.nodes[u].data.b)
            != F::zero()
        {
            return Err(Status::Unbalanced);
        }

        let new_graph = translater(graph);
        let ne = NormalizedNetwork::new(graph);
        self.csr.build(&ne, None, None);
        // self.csr.excesses = new_graph.b.clone().into_boxed_slice();

        for s in 0..self.csr.num_nodes {
            while self.csr.excesses[s] > F::zero() {
                match self.calculate_distance(s) {
                    Some((t, visited, dist, prev)) => {
                        // update potentials
                        for u in 0..self.csr.num_nodes {
                            if visited[u] {
                                self.csr.potentials[u] =
                                    self.csr.potentials[u] - dist[u].unwrap() + dist[t].unwrap();
                            }
                        }
                        // update flow
                        self.update_flow(s, t, prev);
                    }
                    None => break,
                }
            }
        }

        let flows = self.csr.set_flow(graph);

        if self.csr.excesses.iter().all(|&e| e == F::zero()) {
            Ok((0..graph.num_edges()).fold(F::zero(), |cost, edge_id| {
                let edge = graph.get_edge(EdgeId(edge_id));
                cost + edge.data.cost * flows[edge_id]
            }))
        } else {
            Err(Status::Infeasible)
        }
    }
}

impl<F> SuccessiveShortestPath<F>
where
    F: MinimumCostFlowNum,
{
    pub fn solve(
        &mut self,
        graph: &mut Graph<Directed, MinimumCostFlowNode<F>,MinimumCostFlowEdge<F>>,
    ) -> Result<F, Status> {
        <Self as MinimumCostFlowSolver<F>>::solve(self, graph)
    }

    pub fn calculate_distance(
        &mut self,
        s: usize,
    ) -> Option<(usize, Vec<bool>, Vec<Option<F>>, Vec<Option<usize>>)> {
        let mut prev = vec![None; self.csr.num_nodes];
        let mut bh = BinaryHeap::new();
        let mut dist: Vec<Option<F>> = vec![None; self.csr.num_nodes];
        let mut visited = vec![false; self.csr.num_nodes];

        bh.push((Reverse(F::zero()), s));
        dist[s] = Some(F::zero());

        while let Some((d, u)) = bh.pop() {
            if visited[u] {
                continue;
            }
            visited[u] = true;

            if self.csr.excesses[u] < F::zero() {
                return Some((u, visited, dist, prev));
            }

            for edge_id in self.csr.start[u]..self.csr.start[u + 1] {
                if self.csr.residual_capacity(edge_id) == F::zero() {
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
        debug_assert!(self.csr.excesses[s] > F::zero() && self.csr.excesses[t] < F::zero());

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
            debug_assert!(delta > F::zero());
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
