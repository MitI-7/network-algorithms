use crate::{
    algorithms::minimum_cost_flow::{
        MinimumCostFlowNum,
        edge::MinimumCostFlowEdge,
        node::MinimumCostFlowNode,
        normalized_network::NormalizedNetwork,
        residual_network::ResidualNetwork,
        result::MinimumCostFlowResult,
        solver::MinimumCostFlowSolver,
        status::Status,
        validate::{trivial_solution_if_any, validate_balance, validate_infeasible},
    },
    graph::{
        direction::Directed,
        graph::Graph,
        ids::{ArcId, EdgeId},
    },
};
use std::{cmp::Reverse, collections::BinaryHeap};

#[derive(Default)]
pub struct SuccessiveShortestPath<F> {
    rn: ResidualNetwork<F>,
}

impl<F> MinimumCostFlowSolver<F> for SuccessiveShortestPath<F>
where
    F: MinimumCostFlowNum,
{
    fn solve(
        &mut self,
        graph: &mut Graph<Directed, MinimumCostFlowNode<F>, MinimumCostFlowEdge<F>>,
    ) -> Result<MinimumCostFlowResult<F>, Status> {
        self.run(graph)
    }
}

impl<F> SuccessiveShortestPath<F>
where
    F: MinimumCostFlowNum,
{
    pub fn run(
        &mut self,
        graph: &mut Graph<Directed, MinimumCostFlowNode<F>, MinimumCostFlowEdge<F>>,
    ) -> Result<MinimumCostFlowResult<F>, Status> {
        validate_balance(graph)?;
        validate_infeasible(graph)?;

        if let Some(res) = trivial_solution_if_any(graph) {
            return res;
        }

        let nn = NormalizedNetwork::new(graph);
        self.rn.build(&nn, None, None, None);

        for s in 0..self.rn.num_nodes {
            while self.rn.excesses[s] > F::zero() {
                match self.calculate_distance(s) {
                    Some((t, visited, dist, prev)) => {
                        // update potentials
                        for u in 0..self.rn.num_nodes {
                            if visited[u] {
                                self.rn.potentials[u] =
                                    self.rn.potentials[u] - dist[u].unwrap() + dist[t].unwrap();
                            }
                        }
                        // update flow
                        self.update_flow(s, t, prev);
                    }
                    None => break,
                }
            }
        }

        if self.rn.excesses.iter().all(|&e| e == F::zero()) {
            let flows = self.rn.get_flow(graph);
            let objective_value = (0..graph.num_edges()).fold(F::zero(), |cost, edge_id| {
                let edge = graph.get_edge(EdgeId(edge_id));
                cost + edge.data.cost * flows[edge_id]
            });
            Ok(MinimumCostFlowResult {
                objective_value,
                flows,
            })
        } else {
            Err(Status::Infeasible)
        }
    }

    fn calculate_distance(
        &mut self,
        s: usize,
    ) -> Option<(usize, Vec<bool>, Vec<Option<F>>, Vec<Option<ArcId>>)> {
        let mut prev = vec![None; self.rn.num_nodes];
        let mut bh = BinaryHeap::new();
        let mut dist: Vec<Option<F>> = vec![None; self.rn.num_nodes];
        let mut visited = vec![false; self.rn.num_nodes];

        bh.push((Reverse(F::zero()), s));
        dist[s] = Some(F::zero());

        while let Some((d, u)) = bh.pop() {
            if visited[u] {
                continue;
            }
            visited[u] = true;

            if self.rn.excesses[u] < F::zero() {
                return Some((u, visited, dist, prev));
            }

            for arc_id in self.rn.start[u]..self.rn.start[u + 1] {
                let arc_id = ArcId(arc_id);
                if self.rn.residual_capacity(arc_id) == F::zero() {
                    continue;
                }

                let to = self.rn.to[arc_id.index()];
                let new_dist = d.0 + self.rn.reduced_cost(u, arc_id);
                if dist[to].is_none() || dist[to].unwrap() > new_dist {
                    dist[to] = Some(new_dist);
                    prev[to] = Some(arc_id);
                    bh.push((Reverse(new_dist), to));
                }
            }
        }

        None
    }

    fn update_flow(&mut self, s: usize, t: usize, prev: Vec<Option<ArcId>>) {
        debug_assert!(self.rn.excesses[s] > F::zero() && self.rn.excesses[t] < F::zero());

        // calculate delta
        let mut delta = self.rn.excesses[s].min(-self.rn.excesses[t]);
        {
            let mut v = t;
            while let Some(arc_id) = prev[v] {
                delta = delta.min(self.rn.residual_capacity(arc_id));
                let rev = self.rn.rev[arc_id.index()];
                v = self.rn.to[rev];
            }
            delta = delta.min(self.rn.excesses[v]);
            debug_assert_eq!(s, v);
            debug_assert!(delta > F::zero());
        }

        // update flow
        {
            let mut v = t;
            while let Some(arc_id) = prev[v] {
                // push
                let rev = self.rn.rev[arc_id.index()];
                self.rn.flow[arc_id.index()] += delta;
                self.rn.flow[rev] -= delta;
                v = self.rn.to[rev];
            }
            debug_assert_eq!(s, v);
        }

        self.rn.excesses[t] += delta;
        self.rn.excesses[s] -= delta;
    }
}
