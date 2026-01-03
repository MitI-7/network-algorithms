use crate::algorithms::minimum_cost_flow::algorithms::solver::MinimumCostFlowSolver;
use crate::graph::ids::ArcId;
use crate::{
    algorithms::minimum_cost_flow::{
        edge::MinimumCostFlowEdge,
        node::MinimumCostFlowNode,
        normalized_network::NormalizedNetwork,
        residual_network::{ResidualNetwork, construct_extend_network_one_supply_one_demand},
        result::MinimumCostFlowResult,
        status::Status,
        validate::{trivial_solution_if_any, validate_balance, validate_infeasible},
    },
    core::numeric::CostNum,
    graph::{
        direction::Directed,
        graph::Graph,
        ids::{EdgeId, NodeId},
    },
};
use std::collections::{BinaryHeap, VecDeque};
use crate::minimum_cost_flow::algorithms::macros::impl_minimum_cost_flow_solver;

pub struct PrimalDual<F> {
    rn: ResidualNetwork<F>,

    // maximum flow(dinic)
    que: VecDeque<NodeId>,
    distances: Box<[usize]>,
    current_edge: Box<[usize]>,

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
        let rn = ResidualNetwork::new(&nn, Some(&[source, sink]), Some(&artificial_edges), Some(&excess_fix));
        let num_nodes = rn.num_nodes;
        
        PrimalDual {
            rn,
            que: VecDeque::new(),
            distances: vec![0; num_nodes].into_boxed_slice(),
            current_edge: vec![0; num_nodes].into_boxed_slice(),
            source,
            sink,
        }
    }

    fn run(&mut self) -> Result<MinimumCostFlowResult<F>, Status> {
        // validate_balance(graph)?;
        // validate_infeasible(graph)?;

        // if let Some(res) = trivial_solution_if_any(graph) {
        //     return res;
        // }

        while self.rn.excesses[self.source.index()] > F::zero() {
            if !self.dual(self.source, self.sink) {
                break;
            }
            self.primal(self.source, self.sink);
        }

        // graph.remove_artificial_sub_graph(&artificial_nodes, &artificial_edges);
        if self.rn.excesses[self.source.index()] != F::zero() || self.rn.excesses[self.sink.index()] != F::zero() {
            return Err(Status::Infeasible);
        }

        Ok(self.rn.make_minimum_cost_flow_result_in_original_graph())
    }

    // update potentials
    fn dual(&mut self, source: NodeId, sink: NodeId) -> bool {
        assert!(self.rn.excesses[source.index()] > F::zero());

        // calculate the shortest path
        let mut dist: Vec<Option<F>> = vec![None; self.rn.num_nodes];
        let mut visited = vec![false; self.rn.num_nodes];
        {
            let mut bh: BinaryHeap<(F, NodeId)> = BinaryHeap::new();

            bh.push((F::zero(), source));
            dist[source.index()] = Some(F::zero());

            while let Some((mut d, u)) = bh.pop() {
                d = -d;

                if visited[u.index()] {
                    continue;
                }
                visited[u.index()] = true;

                for edge_index in self.rn.neighbors(u) {
                    if self.rn.residual_capacity(edge_index) == F::zero() {
                        continue;
                    }
                    let to = self.rn.to[edge_index.index()];
                    if dist[to.index()].is_none() || dist[to.index()].unwrap() > d + self.rn.reduced_cost(u, edge_index)
                    {
                        dist[to.index()] = Some(d + self.rn.reduced_cost(u, edge_index));
                        bh.push((-dist[to.index()].unwrap(), to));
                    }
                }
            }
        }

        // update potentials
        for u in 0..self.rn.num_nodes {
            if visited[u] {
                self.rn.potentials[u] -= dist[u].unwrap();
            }
        }

        visited[sink.index()]
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
                if self.rn.flow[arc_id.index()] > F::zero()
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
                self.rn.flow[arc_id.index()] += d;
                self.rn.flow[rev.index()] -= d;

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
}

impl_minimum_cost_flow_solver!(PrimalDual, run);