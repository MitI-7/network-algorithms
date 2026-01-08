use crate::{
    algorithms::minimum_cost_flow::{
        edge::MinimumCostFlowEdge,
        node::MinimumCostFlowNode,
        normalized_network::NormalizedNetwork,
        residual_network::{ResidualNetwork, construct_extend_network_feasible_solution},
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

// O(nU * (m + n) log n)
pub struct OutOfKilter<F> {
    rn: ResidualNetwork<F>,
}

impl<F> OutOfKilter<F>
where
    F: CostNum,
{
    fn new(graph: &Graph<Directed, MinimumCostFlowNode<F>, MinimumCostFlowEdge<F>>) -> Self {
        let nn = NormalizedNetwork::new(graph);

        let (root, artificial_edges, initial_flows, fix_excesses) = construct_extend_network_feasible_solution(&nn);
        let rn = ResidualNetwork::new(
            &nn,
            Some(&[root]),
            Some(&artificial_edges),
            Some(&initial_flows),
            Some(&fix_excesses),
        );
        OutOfKilter { rn }
    }

    fn run(&mut self) -> Result<F, Status> {
        validate_balance(&self.rn)?;
        validate_infeasible(&self.rn)?;
        if let Some(res) = trivial_solution_if_any(&self.rn) {
            return res;
        }

        let mut out_of_kilter_edges = Vec::new();
        for arc_id in 0..self.rn.to.len() {
            let arc_id = ArcId(arc_id);
            let rev = self.rn.rev[arc_id.index()];
            let p = self.rn.to[rev.index()];
            if self.kilter_number(p, arc_id) != F::zero() {
                let q = self.rn.to[arc_id.index()];
                out_of_kilter_edges.push((p, q, arc_id));
            }
        }

        'outer: for (p, q, edge_id) in out_of_kilter_edges {
            while self.kilter_number(p, edge_id) > F::zero() {
                let (dist, prev) = self.shortest_path(q);
                if prev[p.index()].is_none() {
                    break 'outer;
                }

                // update potentials
                for u in 0..self.rn.num_nodes {
                    if let Some(d) = dist[u] {
                        self.rn.potentials[u] -= d;
                    }
                }

                // update flow
                if self.rn.reduced_cost(p, edge_id) < F::zero() {
                    self.update_flow_in_cycle(q, edge_id, prev);
                }
            }
        }

        if (self.rn.num_edges_original_graph..self.rn.num_edges)
            .into_iter()
            .all(|edge_id| {
                let arc_id = self.rn.edge_id_to_arc_id[edge_id];
                self.rn.residual_capacity[arc_id.index()] == self.rn.upper[arc_id.index()]
            })
        {
            Ok(self.rn.calculate_objective_value_in_original_graph())
        } else {
            Err(Status::Infeasible)
        }
    }

    fn kilter_number(&self, u: NodeId, arc_id: ArcId) -> F {
        if self.rn.reduced_cost(u, arc_id) >= F::zero() {
            F::zero()
        } else {
            self.rn.residual_capacity(arc_id)
        }
    }

    fn shortest_path(&mut self, s: NodeId) -> (Vec<Option<F>>, Vec<Option<ArcId>>) {
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

            for arc_id in self.rn.start[u.index()]..self.rn.start[u.index() + 1] {
                let arc_id = ArcId(arc_id);
                if self.rn.residual_capacity(arc_id) <= F::zero() {
                    continue;
                }

                let to = self.rn.to[arc_id.index()];
                let new_dist = d.0 + self.rn.reduced_cost(u, arc_id).max(F::zero());
                if dist[to.index()].is_none() || dist[to.index()].unwrap() > new_dist {
                    dist[to.index()] = Some(new_dist);
                    prev[to.index()] = Some(arc_id);
                    bh.push((Reverse(new_dist), to));
                }
            }
        }

        (dist, prev)
    }

    fn update_flow_in_cycle(&mut self, q: NodeId, arc_id: ArcId, mut prev: Vec<Option<ArcId>>) {
        prev[q.index()] = Some(arc_id); // p -> q

        // calculate delta
        let mut delta = self.rn.residual_capacity(arc_id);
        let mut v = q;
        while let Some(arc_idx) = prev[v.index()] {
            delta = delta.min(self.rn.residual_capacity(arc_idx));
            let rev = self.rn.rev[arc_idx.index()];
            v = self.rn.to[rev.index()];
            if v == q {
                break;
            }
        }

        // update flow
        let mut v = q;
        while let Some(arc_id) = prev[v.index()] {
            let rev = self.rn.rev[arc_id.index()];
            v = self.rn.to[rev.index()];
            self.rn.push_flow(v, arc_id, delta);
            if v == q {
                break;
            }
        }
    }

    fn make_minimum_cost_flow_in_original_graph(&self) -> Vec<F> {
        self.rn.make_minimum_cost_flow_in_original_graph()
    }

    fn flow(&self, edge_id: EdgeId) -> Option<F> {
        self.rn.flow(edge_id)
    }
}

impl_minimum_cost_flow_solver!(OutOfKilter, run);
