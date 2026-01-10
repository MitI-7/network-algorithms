use crate::{
    algorithms::minimum_cost_flow::{
        edge::MinimumCostFlowEdge,
        extend_network::construct_extend_network_one_supply_one_demand,
        node::MinimumCostFlowNode,
        normalized_network::NormalizedNetwork,
        residual_network::ResidualNetwork,
        solvers::{macros::impl_minimum_cost_flow_solver, solver::MinimumCostFlowSolver},
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
use std::{cmp::Reverse, collections::BinaryHeap};

pub struct SuccessiveShortestPath<F> {
    rn: ResidualNetwork<F>,
    source: NodeId,
}

impl<F> SuccessiveShortestPath<F>
where
    F: CostNum,
{
    pub fn new(graph: &Graph<Directed, MinimumCostFlowNode<F>, MinimumCostFlowEdge<F>>) -> Self {
        let nn = NormalizedNetwork::new(graph);
        let (source, sink, artificial_edges, excess_fix) = construct_extend_network_one_supply_one_demand(&nn);
        let rn = ResidualNetwork::new(&nn, Some(&[source, sink]), Some(&artificial_edges), None, Some(&excess_fix));
        Self { rn, source }
    }

    fn run(&mut self) -> Result<F, Status> {
        validate_balance(&self.rn)?;
        validate_infeasible(&self.rn)?;

        if let Some(res) = trivial_solution_if_any(&self.rn) {
            return res;
        }

        while self.rn.excesses[self.source.index()] > F::zero() {
            match self.calculate_distance(self.source) {
                Some((t, visited, dist, prev)) => {
                    // update potentials
                    for u in 0..self.rn.num_nodes {
                        if visited[u] {
                            self.rn.potentials[u] = self.rn.potentials[u] - dist[u].unwrap() + dist[t.index()].unwrap();
                        }
                    }
                    // update flow
                    self.update_flow(self.source, t, prev);
                }
                None => break,
            }
        }

        if self.rn.have_excess() {
            Err(Status::Infeasible)
        } else {
            Ok(self.rn.calculate_objective_value_original_graph())
        }
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

            for arc_id in (self.rn.start[u.index()]..self.rn.start[u.index() + 1]).map(ArcId) {
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
                self.rn.residual_capacity[arc_id.index()] -= delta;
                self.rn.residual_capacity[rev.index()] += delta;
                v = self.rn.to[rev.index()];
            }
            debug_assert_eq!(s, v);
        }

        self.rn.excesses[t.index()] += delta;
        self.rn.excesses[s.index()] -= delta;
    }

    fn flow(&self, edge_id: EdgeId) -> Option<F> {
        self.rn.flow_original_graph(edge_id)
    }

    fn flows(&self) -> Vec<F> {
        self.rn.flows_original_graph()
    }

    fn potential(&self, node_id: NodeId) -> Option<F> {
        self.rn.potential_original_graph(node_id)
    }

    fn potentials(&self) -> Vec<F> {
        self.rn.potentials_original_graph()
    }
}

impl_minimum_cost_flow_solver!(SuccessiveShortestPath, run);
