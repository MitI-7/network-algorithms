use crate::algorithms::minimum_cost_flow::algorithms::solver::MinimumCostFlowSolver;
use crate::minimum_cost_flow::algorithms::macros::impl_minimum_cost_flow_solver;
use crate::minimum_cost_flow::prelude::PrimalDual;
use crate::minimum_cost_flow::residual_network::construct_extend_network_one_supply_one_demand;
use crate::{
    algorithms::minimum_cost_flow::{
        edge::MinimumCostFlowEdge,
        node::MinimumCostFlowNode,
        normalized_network::NormalizedNetwork,
        residual_network::ResidualNetwork,
        result::MinimumCostFlowResult,
        status::Status,
        validate::{trivial_solution_if_any, validate_balance, validate_infeasible},
    },
    core::numeric::CostNum,
    graph::{
        direction::Directed,
        graph::Graph,
        ids::{ArcId, EdgeId, NodeId},
    },
};
use std::collections::VecDeque;
use std::{cmp::Reverse, collections::BinaryHeap};

#[derive(Default)]
pub struct SuccessiveShortestPath<F> {
    rn: ResidualNetwork<F>,
}

impl<F> SuccessiveShortestPath<F>
where
    F: CostNum,
{
    pub fn new(graph: &Graph<Directed, MinimumCostFlowNode<F>, MinimumCostFlowEdge<F>>) -> Self {
        let nn = NormalizedNetwork::new(graph);

        // transforms the minimum cost flow problem into a problem with a single excess node and a single deficit node.
        // let (source, sink, artificial_edges, excess_fix) = construct_extend_network_one_supply_one_demand(&nn);
        let rn = ResidualNetwork::new(&nn, None, None, None);

        Self { rn }
    }

    fn run(&mut self) -> Result<MinimumCostFlowResult<F>, Status> {
        // validate_balance(graph)?;
        // validate_infeasible(graph)?;

        // if let Some(res) = trivial_solution_if_any(graph) {
        //     return res;
        // }

        // let nn = NormalizedNetwork::new(graph);
        // self.rn.build(&nn, None, None, None);

        for s in 0..self.rn.num_nodes {
            let s = NodeId(s);
            while self.rn.excesses[s.index()] > F::zero() {
                match self.calculate_distance(s) {
                    Some((t, visited, dist, prev)) => {
                        // update potentials
                        for u in 0..self.rn.num_nodes {
                            if visited[u] {
                                self.rn.potentials[u] =
                                    self.rn.potentials[u] - dist[u].unwrap() + dist[t.index()].unwrap();
                            }
                        }
                        // update flow
                        self.update_flow(s, t, prev);
                    }
                    None => break,
                }
            }
        }

        if self.rn.excesses.iter().any(|&f| f > F::zero()) {
            return Err(Status::Infeasible);
        }

        Ok(self.rn.make_minimum_cost_flow_result_in_original_graph())
    }

    fn calculate_distance(&mut self, s: NodeId) -> Option<(NodeId, Vec<bool>, Vec<Option<F>>, Vec<Option<ArcId>>)> {
        let mut prev = vec![None; self.rn.num_nodes];
        let mut bh = BinaryHeap::new();
        let mut dist: Vec<Option<F>> = vec![None; self.rn.num_nodes];
        let mut visited = vec![false; self.rn.num_nodes];

        bh.push((Reverse(F::zero()), s));
        dist[s.index()] = Some(F::zero());

        while let Some((d, u)) = bh.pop() {
            if visited[u.index()] {
                continue;
            }
            visited[u.index()] = true;

            if self.rn.excesses[u.index()] < F::zero() {
                return Some((u, visited, dist, prev));
            }

            for arc_id in self.rn.start[u.index()]..self.rn.start[u.index() + 1] {
                let arc_id = ArcId(arc_id);
                if self.rn.residual_capacity(arc_id) == F::zero() {
                    continue;
                }

                let to = self.rn.to[arc_id.index()];
                let new_dist = d.0 + self.rn.reduced_cost(u, arc_id);
                if dist[to.index()].is_none() || dist[to.index()].unwrap() > new_dist {
                    dist[to.index()] = Some(new_dist);
                    prev[to.index()] = Some(arc_id);
                    bh.push((Reverse(new_dist), to));
                }
            }
        }

        None
    }

    fn update_flow(&mut self, s: NodeId, t: NodeId, prev: Vec<Option<ArcId>>) {
        debug_assert!(self.rn.excesses[s.index()] > F::zero() && self.rn.excesses[t.index()] < F::zero());

        // calculate delta
        let mut delta = self.rn.excesses[s.index()].min(-self.rn.excesses[t.index()]);
        {
            let mut v = t;
            while let Some(arc_id) = prev[v.index()] {
                delta = delta.min(self.rn.residual_capacity(arc_id));
                let rev = self.rn.rev[arc_id.index()];
                v = self.rn.to[rev.index()];
            }
            delta = delta.min(self.rn.excesses[v.index()]);
            debug_assert_eq!(s, v);
            debug_assert!(delta > F::zero());
        }

        // update flow
        {
            let mut v = t;
            while let Some(arc_id) = prev[v.index()] {
                // push
                let rev = self.rn.rev[arc_id.index()];
                self.rn.flow[arc_id.index()] += delta;
                self.rn.flow[rev.index()] -= delta;
                v = self.rn.to[rev.index()];
            }
            debug_assert_eq!(s, v);
        }

        self.rn.excesses[t.index()] += delta;
        self.rn.excesses[s.index()] -= delta;
    }
}

impl_minimum_cost_flow_solver!(SuccessiveShortestPath, run);
