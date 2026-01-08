use crate::{
    algorithms::minimum_cost_flow::{
        edge::MinimumCostFlowEdge,
        node::MinimumCostFlowNode,
        normalized_network::NormalizedNetwork,
        residual_network::{ResidualNetwork, construct_extend_network_one_supply_one_demand},
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
use std::cmp::Reverse;
use std::collections::{BinaryHeap, VecDeque};

pub struct PrimalDual<F> {
    rn: ResidualNetwork<F>,

    // maximum flow(dinic)
    que: VecDeque<NodeId>,
    distances: Box<[usize]>,
    current_edge: Box<[usize]>,

    // working
    dist: Box<[Option<F>]>,
    visited: Box<[bool]>,

    // extended network
    source: NodeId,
    sink: NodeId,
}

impl<F> PrimalDual<F>
where
    F: CostNum,
{
    pub fn new(graph: &Graph<Directed, MinimumCostFlowNode<F>, MinimumCostFlowEdge<F>>) -> Self {
        let nn = NormalizedNetwork::new(graph);

        // transforms the minimum cost flow problem into a problem with a single excess node and a single deficit node.
        let (source, sink, artificial_edges, excess_fix) = construct_extend_network_one_supply_one_demand(&nn);
        let rn = ResidualNetwork::new(&nn, Some(&[source, sink]), Some(&artificial_edges), None, Some(&excess_fix));
        let num_nodes = rn.num_nodes;

        Self {
            rn,
            que: VecDeque::new(),
            distances: vec![0; num_nodes].into_boxed_slice(),
            current_edge: vec![0; num_nodes].into_boxed_slice(),
            dist: vec![None; num_nodes].into_boxed_slice(),
            visited: vec![false; num_nodes].into_boxed_slice(),
            source,
            sink,
        }
    }

    fn run(&mut self) -> Result<F, Status> {
        validate_balance(&self.rn)?;
        validate_infeasible(&self.rn)?;

        if let Some(res) = trivial_solution_if_any(&self.rn) {
            return res;
        }

        while self.rn.excesses[self.source.index()] > F::zero() {
            if !self.dual(self.source, self.sink) {
                break;
            }
            self.primal(self.source, self.sink);
        }

        if self.rn.excesses[self.source.index()] != F::zero() || self.rn.excesses[self.sink.index()] != F::zero() {
            return Err(Status::Infeasible);
        }

        Ok(self.rn.calculate_objective_value_in_original_graph())
    }

    // update potentials
    fn dual(&mut self, source: NodeId, sink: NodeId) -> bool {
        assert!(self.rn.excesses[source.index()] > F::zero());

        // calculate the shortest path
        self.dist.fill(None);
        self.visited.fill(false);
        {
            let mut bh: BinaryHeap<(Reverse<F>, NodeId)> = BinaryHeap::new();

            bh.push((Reverse(F::zero()), source));
            self.dist[source.index()] = Some(F::zero());

            while let Some((d, u)) = bh.pop() {
                if self.visited[u.index()] {
                    continue;
                }
                self.visited[u.index()] = true;

                for edge_index in self.rn.neighbors(u) {
                    if self.rn.residual_capacity(edge_index) == F::zero() {
                        continue;
                    }
                    let to = self.rn.to[edge_index.index()];
                    if self.dist[to.index()].is_none()
                        || self.dist[to.index()].unwrap() > d.0 + self.rn.reduced_cost(u, edge_index)
                    {
                        self.dist[to.index()] = Some(d.0 + self.rn.reduced_cost(u, edge_index));
                        bh.push((Reverse(self.dist[to.index()].unwrap()), to));
                    }
                }
            }
        }

        // update potentials
        for u in 0..self.rn.num_nodes {
            if self.visited[u] {
                self.rn.potentials[u] -= self.dist[u].unwrap();
            }
        }

        self.visited[sink.index()]
    }

    fn primal(&mut self, source: NodeId, sink: NodeId) {
        assert!(self.rn.excesses[source.index()] > F::zero() && self.rn.excesses[sink.index()] < F::zero());

        let mut flow = F::zero();
        while self.rn.excesses[source.index()] > F::zero() {
            self.update_distances(source, sink);

            // no s-t path
            if self.distances[source.index()] >= self.rn.num_nodes {
                break;
            }

            self.current_edge
                .iter_mut()
                .enumerate()
                .for_each(|(u, e)| *e = self.rn.start[u]);
            match self.dfs(source, sink, self.rn.excesses[source.index()]) {
                Some(delta) => flow += delta,
                None => break,
            }
        }
        self.rn.excesses[source.index()] -= flow;
        self.rn.excesses[sink.index()] += flow;
    }

    // O(n + m)
    // calculate the distance from u to sink in the residual network
    // if such a path does not exist, distance[u] becomes self.num_nodes
    fn update_distances(&mut self, source: NodeId, sink: NodeId) {
        self.que.clear();
        self.que.push_back(sink);
        self.distances.fill(self.rn.num_nodes);
        self.distances[sink.index()] = 0;

        while let Some(v) = self.que.pop_front() {
            for arc_id in self.rn.neighbors(v) {
                // e.to -> v
                let to = self.rn.to[arc_id.index()];
                let rev_arc_id = self.rn.rev[arc_id.index()];
                if self.rn.residual_capacity[rev_arc_id.index()] > F::zero()
                    && self.distances[to.index()] == self.rn.num_nodes
                    && self.rn.reduced_cost_rev(v, arc_id) == F::zero()
                {
                    self.distances[to.index()] = self.distances[v.index()] + 1;
                    if to != source {
                        self.que.push_back(to);
                    }
                }
            }
        }
    }

    fn dfs(&mut self, u: NodeId, sink: NodeId, upper: F) -> Option<F> {
        if u == sink {
            return Some(upper);
        }

        let mut res = F::zero();
        for arc_id in self.current_edge[u.index()]..self.rn.start[u.index() + 1] {
            let arc_id = ArcId(arc_id);
            self.current_edge[u.index()] = arc_id.index();

            if !self.is_admissible_edge(u, arc_id) || self.rn.reduced_cost(u, arc_id) != F::zero() {
                continue;
            }

            let v = self.rn.to[arc_id.index()];
            let residual_capacity = self.rn.residual_capacity(arc_id);
            if let Some(d) = self.dfs(v, sink, residual_capacity.min(upper - res)) {
                let rev = self.rn.rev[arc_id.index()];

                // update flow
                self.rn.residual_capacity[arc_id.index()] -= d;
                self.rn.residual_capacity[rev.index()] += d;

                res += d;
                if res == upper {
                    return Some(res);
                }
            }
        }
        self.current_edge[u.index()] = self.rn.start[u.index() + 1];
        self.distances[u.index()] = self.rn.num_nodes;

        Some(res)
    }

    #[inline]
    fn is_admissible_edge(&self, from: NodeId, arc_id: ArcId) -> bool {
        self.rn.residual_capacity(arc_id) > F::zero()
            && self.distances[from.index()] == self.distances[self.rn.to[arc_id.index()].index()] + 1
    }

    fn make_minimum_cost_flow_in_original_graph(&self) -> Vec<F> {
        self.rn.make_minimum_cost_flow_in_original_graph()
    }

    fn flow(&self, edge_id: EdgeId) -> Option<F> {
        self.rn.flow(edge_id)
    }

    fn flows(&self) -> Vec<F> {
        self.rn.flows()
    }
    
    fn potential(&self, node_id: NodeId) -> Option<F> {
        self.rn.potential(node_id)
    }

    fn potentials(&self) -> Vec<F> {
        self.rn.potentials()
    }
}

impl_minimum_cost_flow_solver!(PrimalDual, run);
