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
        ids::{ArcId, EdgeId, INVALID_ARC_ID, INVALID_NODE_ID, NodeId},
    },
};

impl<F> CycleCanceling<F>
where
    F: CostNum,
{
    pub fn new(graph: &Graph<Directed, MinimumCostFlowNode<F>, MinimumCostFlowEdge<F>>) -> Self {
        let nn = NormalizedNetwork::new(graph);

        let (root, artificial_edges, initial_flows, excess_fix) = construct_extend_network_feasible_solution(&nn);
        let rn =
            ResidualNetwork::new(&nn, Some(&[root]), Some(&artificial_edges), Some(&initial_flows), Some(&excess_fix));
        let num_nodes = rn.num_nodes;
        CycleCanceling {
            rn,
            dist: vec![F::zero(); num_nodes].into_boxed_slice(),
            visited: vec![false; num_nodes].into_boxed_slice(),
        }
    }

    fn run(&mut self) -> Result<F, Status> {
        validate_balance(&self.rn)?;
        validate_infeasible(&self.rn)?;

        if let Some(res) = trivial_solution_if_any(&self.rn) {
            return res;
        }

        let mut prev = vec![(INVALID_NODE_ID, INVALID_ARC_ID); self.rn.num_nodes];
        while let Some(start) = self.find_negative_cycle(&mut prev) {
            let (mut v, idx) = prev[start.index()];
            let mut delta = self.rn.residual_capacity(idx);
            let mut cycle = vec![idx];
            while v != start {
                let (u, idx) = prev[v.index()];
                cycle.push(idx);
                delta = delta.min(self.rn.residual_capacity(idx));
                v = u;
            }
            assert!(delta > F::zero());

            for idx in cycle {
                let rev = self.rn.rev[idx.index()];
                self.rn.residual_capacity[idx.index()] -= delta;
                self.rn.residual_capacity[rev.index()] += delta;
            }
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

    fn find_negative_cycle(&mut self, prev: &mut [(NodeId, ArcId)]) -> Option<NodeId> {
        let mut start = INVALID_NODE_ID;
        self.dist.fill(F::zero());
        for _ in 0..self.rn.num_nodes {
            let mut updated = false;
            for u in (0..self.rn.num_nodes).map(NodeId) {
                for arc_id in self.rn.neighbors(u) {
                    let to = self.rn.to[arc_id.index()];
                    let cost = self.rn.cost[arc_id.index()];
                    if self.rn.residual_capacity(arc_id) > F::zero()
                        && self.dist[u.index()] + cost < self.dist[to.index()]
                    {
                        self.dist[to.index()] = self.dist[u.index()] + cost;
                        prev[to.index()] = (u, arc_id);
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
        self.visited.fill(false);
        loop {
            let (u, _) = prev[v.index()];
            if self.visited[u.index()] {
                return Some(v);
            }
            self.visited[u.index()] = true;
            v = u;
        }
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

pub struct CycleCanceling<F> {
    rn: ResidualNetwork<F>,
    dist: Box<[F]>,
    visited: Box<[bool]>,
}

impl_minimum_cost_flow_solver!(CycleCanceling, run);
