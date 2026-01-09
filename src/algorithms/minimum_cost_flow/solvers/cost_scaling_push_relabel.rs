use crate::{
    algorithms::minimum_cost_flow::{
        edge::MinimumCostFlowEdge,
        extend_network::construct_extend_network_feasible_solution,
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
use num_traits::FromPrimitive;
use std::collections::VecDeque;

pub struct CostScalingPushRelabel<F> {
    rn: ResidualNetwork<F>,
    active_nodes: VecDeque<NodeId>,
    current_arc: Vec<usize>,
    alpha: F,
}

#[allow(dead_code)]
impl<F> CostScalingPushRelabel<F>
where
    F: CostNum + FromPrimitive,
{
    pub fn new(graph: &Graph<Directed, MinimumCostFlowNode<F>, MinimumCostFlowEdge<F>>) -> Self {
        let nn = NormalizedNetwork::new(graph);
        let (root, artificial_edges, initial_flows, fix_excesses) = construct_extend_network_feasible_solution(&nn);
        let rn = ResidualNetwork::new(
            &nn,
            Some(&[root]),
            Some(&artificial_edges),
            Some(&initial_flows),
            Some(&fix_excesses),
        );
        Self {
            rn,
            active_nodes: VecDeque::new(),
            current_arc: Vec::new(),
            alpha: F::from_i32(16).expect("cannot represent 16 in F"),
        }
    }

    // scaling_factor: it was usually between 8 and 24. default scaling factor is 16
    // pub fn new(scaling_factor: F) -> Self {
    //     assert!(scaling_factor > F::one());
    //     Self { rn: CSR::default(), active_nodes: VecDeque::new(), current_edge: Vec::new(), alpha: scaling_factor }
    // }

    fn run(&mut self) -> Result<F, Status> {
        validate_balance(&self.rn)?;
        validate_infeasible(&self.rn)?;

        if let Some(res) = trivial_solution_if_any(&self.rn) {
            return res;
        }

        self.current_arc.resize(self.rn.num_nodes, 0);
        let gamma = self.rn.cost.iter().map(|&c| c).max().unwrap_or(F::one()); // all edge costs are non-negative
        let cost_scaling_factor =
            self.alpha * F::from_usize(self.rn.num_nodes).expect("cannot represent num_nodes in F");
        let mut epsilon = F::one().max(gamma * cost_scaling_factor);

        // scale cost
        for i in 0..self.rn.cost.len() {
            self.rn.cost[i] = self.rn.cost[i] * cost_scaling_factor;
        }

        loop {
            epsilon = F::one().max(epsilon / self.alpha);
            self.refine(epsilon);
            if epsilon == F::one() {
                break;
            }
        }
        // unscale cost
        for i in 0..self.rn.cost.len() {
            self.rn.cost[i] = self.rn.cost[i] / cost_scaling_factor;
        }

        if (self.rn.num_edges_original_graph..self.rn.num_edges)
            .into_iter()
            .all(|edge_id| {
                let arc_id = self.rn.edge_id_to_arc_id[edge_id];
                self.rn.residual_capacity[arc_id.index()] == self.rn.upper[arc_id.index()]
            })
        {
            Ok(self.rn.calculate_objective_value_original_graph())
        } else {
            Err(Status::Infeasible)
        }
    }

    // make epsilon-optimal flow
    fn refine(&mut self, epsilon: F) {
        // make 0-optimal pseudo flow
        for u in (0..self.rn.num_nodes).map(NodeId) {
            for arc_id in (self.rn.start[u.index()]..self.rn.start[u.index() + 1]).map(ArcId) {
                let reduced_cost = self.rn.reduced_cost(u, arc_id);
                if reduced_cost < F::zero() {
                    self.rn.push_flow(u, arc_id, self.rn.residual_capacity(arc_id));
                    // debug_assert!(self.rn.flow[arc_id] == self.rn.upper[arc_id]);
                } else if reduced_cost > F::zero() {
                    let f = self.rn.upper[arc_id.index()] - self.rn.residual_capacity[arc_id.index()];
                    self.rn.push_flow(u, arc_id, -f);
                    // debug_assert!(self.rn.flow[arc_id] == F::zero());
                }
            }
        }

        self.current_arc
            .iter_mut()
            .enumerate()
            .for_each(|(u, e)| *e = self.rn.start[u]);

        debug_assert_eq!(self.active_nodes.len(), 0);
        self.active_nodes.extend(
            (0..self.rn.num_nodes)
                .map(NodeId)
                .filter(|&u| self.rn.excesses[u.index()] > F::zero()),
        );

        // 0-optimal pseudo flow -> epsilon-optimal feasible flow
        while let Some(u) = self.active_nodes.pop_back() {
            self.discharge(u, epsilon);
        }
    }

    fn discharge(&mut self, u: NodeId, epsilon: F) {
        while self.rn.excesses[u.index()] > F::zero() {
            self.push(u, epsilon);

            if self.rn.excesses[u.index()] == F::zero() {
                break;
            }

            self.relabel(u, epsilon);
        }
    }

    fn is_admissible(&self, u: NodeId, arc_id: ArcId, _epsilon: F) -> bool {
        self.rn.reduced_cost(u, arc_id) < F::zero()
    }

    fn push(&mut self, u: NodeId, epsilon: F) {
        debug_assert!(self.rn.excesses[u.index()] > F::zero());

        for arc_id in (self.rn.start[u.index()]..self.rn.start[u.index() + 1]).map(ArcId) {
            let to = self.rn.to[arc_id.index()];
            if self.rn.residual_capacity(arc_id) <= F::zero() {
                continue;
            }

            if !self.is_admissible(u, arc_id, epsilon) {
                continue;
            }

            if !self.look_ahead(to, epsilon) {
                if !self.is_admissible(u, arc_id, epsilon) {
                    continue;
                }
            }

            let flow = self.rn.residual_capacity(arc_id).min(self.rn.excesses[u.index()]);
            self.rn.push_flow(u, arc_id, flow);

            if self.rn.excesses[to.index()] > F::zero() && self.rn.excesses[to.index()] <= flow {
                self.active_nodes.push_back(to);
            }

            if self.rn.excesses[u.index()] == F::zero() {
                self.current_arc[u.index()] = arc_id.index();
                return;
            }
        }

        // node has no admissible edge
        self.current_arc[u.index()] = self.rn.start[u.index()];
    }

    fn relabel(&mut self, u: NodeId, epsilon: F) {
        let guaranteed_new_potential = self.rn.potentials[u.index()] + epsilon;

        let mut mini_potential = None;
        let mut previous_mini_potential = None;
        let mut current_edges_for_u = ArcId(0);

        for arc_id in (self.rn.start[u.index()]..self.rn.start[u.index() + 1]).map(ArcId) {
            if self.rn.residual_capacity(arc_id) <= F::zero() {
                continue;
            }

            let to = self.rn.to[arc_id.index()];
            let cost = self.rn.cost[arc_id.index()];

            let new_potential = self.rn.potentials[to.index()] + cost;
            if mini_potential.is_none() || new_potential < mini_potential.unwrap() {
                // adding epsilon creates an admissible edge
                if new_potential < guaranteed_new_potential {
                    self.rn.potentials[u.index()] = guaranteed_new_potential;
                    self.current_arc[u.index()] = arc_id.index();
                    return;
                }

                previous_mini_potential = mini_potential;
                mini_potential = Some(new_potential);
                current_edges_for_u = arc_id;
            }
        }

        // increasing the potential cannot create an admissible edge.
        if mini_potential.is_none() {
            if self.rn.excesses[u.index()] != F::zero() {
                return;
            } else {
                // the potential can be lowered as much as desired, but guaranteed_new_potential is applied
                self.rn.potentials[u.index()] = guaranteed_new_potential;
                self.current_arc[u.index()] = 0;
            }
            return;
        }

        // adding more than epsilon creates an admissible edge
        let new_potential = mini_potential.unwrap() + epsilon;
        self.rn.potentials[u.index()] = new_potential;

        self.current_arc[u.index()] =
            if previous_mini_potential.is_none() || previous_mini_potential.unwrap() >= new_potential {
                current_edges_for_u.index()
            } else {
                self.rn.start[u.index()]
            };
    }

    fn look_ahead(&mut self, u: NodeId, epsilon: F) -> bool {
        if self.rn.excesses[u.index()] < F::zero() {
            return true;
        }

        // search admissible edge
        for arc_id in (self.current_arc[u.index()]..self.rn.start[u.index() + 1]).map(ArcId) {
            if self.rn.residual_capacity(arc_id) <= F::zero() {
                continue;
            }

            if self.is_admissible(u, arc_id, epsilon) {
                self.current_arc[u.index()] = arc_id.index();
                return true;
            }
        }

        self.relabel(u, epsilon);
        false
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
        let n = self.rn.num_nodes;
        let mut dist = vec![F::zero(); n]; // スーパーソースを全頂点に0で繋ぐのと等価

        // Bellman-Ford: 残余容量>0の残余辺だけで最短距離 dist を計算
        for _ in 0..n.saturating_sub(1) {
            let mut updated = false;
            for u in (0..n).map(NodeId) {
                for e in self.rn.neighbors(u) {
                    if self.rn.residual_capacity[e.index()] > F::zero() {
                        let v = self.rn.to[e.index()];
                        let cand = dist[u.index()] + self.rn.cost[e.index()];
                        if cand < dist[v.index()] {
                            dist[v.index()] = cand;
                            updated = true;
                        }
                    }
                }
            }
            if !updated { break; }
        }

        // ここが重要：check_optimality の r = c - π(u) + π(v) に合わせて π = -dist
        dist.into_iter().map(|d| -d).collect()
    }
}

impl_minimum_cost_flow_solver!(CostScalingPushRelabel, run, FromPrimitive);
