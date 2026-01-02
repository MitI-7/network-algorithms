use crate::graph::ids::ArcId;
use crate::{
    algorithms::minimum_cost_flow::{
        MinimumCostFlowNum,
        edge::MinimumCostFlowEdge,
        node::MinimumCostFlowNode,
        normalized_network::NormalizedNetwork,
        residual_network::{ResidualNetwork, construct_extend_network_one_supply_one_demand},
        result::MinimumCostFlowResult,
        solver::MinimumCostFlowSolver,
        status::Status,
        validate::{trivial_solution_if_any, validate_balance, validate_infeasible},
    },
    graph::{direction::Directed, graph::Graph, ids::EdgeId},
};
use std::collections::{BinaryHeap, VecDeque};

#[derive(Default)]
pub struct PrimalDual<F> {
    rn: ResidualNetwork<F>,

    // maximum flow(dinic)
    que: VecDeque<usize>,
    distances: Vec<usize>,
    current_edge: Vec<usize>,
}

impl<F> MinimumCostFlowSolver<F> for PrimalDual<F>
where
    F: MinimumCostFlowNum,
{
    fn solve(
        &mut self,
        graph: &Graph<Directed, MinimumCostFlowNode<F>, MinimumCostFlowEdge<F>>,
    ) -> Result<MinimumCostFlowResult<F>, Status> {
        self.run(graph)
    }
}

impl<F> PrimalDual<F>
where
    F: MinimumCostFlowNum,
{
    pub fn run(
        &mut self,
        graph: &Graph<Directed, MinimumCostFlowNode<F>, MinimumCostFlowEdge<F>>,
    ) -> Result<MinimumCostFlowResult<F>, Status> {
        validate_balance(graph)?;
        validate_infeasible(graph)?;

        if let Some(res) = trivial_solution_if_any(graph) {
            return res;
        }

        let nn = NormalizedNetwork::new(graph);

        // transforms the minimum cost flow problem into a problem with a single excess node and a single deficit node.
        let (source, sink, artificial_edges, excess_fix) =
            construct_extend_network_one_supply_one_demand(&nn);
        self.rn.build(
            &nn,
            Some(&[source, sink]),
            Some(&artificial_edges),
            Some(&excess_fix),
        );

        self.distances.resize(self.rn.num_nodes, 0);
        self.current_edge.resize(self.rn.num_nodes, 0);

        while self.rn.excesses[source.index()] > F::zero() {
            if !self.dual(source.index(), sink.index()) {
                break;
            }
            self.primal(source.index(), sink.index());
        }

        let flows = self.rn.get_flow(graph);

        // graph.remove_artificial_sub_graph(&artificial_nodes, &artificial_edges);
        if self.rn.excesses[source.index()] != F::zero()
            || self.rn.excesses[sink.index()] != F::zero()
        {
            return Err(Status::Infeasible);
        }

        Ok(MinimumCostFlowResult {
            objective_value: (0..graph.num_edges()).fold(F::zero(), |cost, edge_id| {
                let edge = graph.get_edge(EdgeId(edge_id));
                cost + edge.data.cost * flows[edge_id]
            }),
            flows,
        })
    }

    // update potentials
    fn dual(&mut self, source: usize, sink: usize) -> bool {
        assert!(self.rn.excesses[source] > F::zero());

        // calculate the shortest path
        let mut dist: Vec<Option<F>> = vec![None; self.rn.num_nodes];
        let mut visited = vec![false; self.rn.num_nodes];
        {
            let mut bh: BinaryHeap<(F, usize)> = BinaryHeap::new();

            bh.push((F::zero(), source));
            dist[source] = Some(F::zero());

            while let Some((mut d, u)) = bh.pop() {
                d = -d;

                if visited[u] {
                    continue;
                }
                visited[u] = true;

                for edge_index in self.rn.neighbors(u) {
                    if self.rn.residual_capacity(edge_index) == F::zero() {
                        continue;
                    }
                    let to = self.rn.to[edge_index.index()];
                    if dist[to].is_none()
                        || dist[to].unwrap() > d + self.rn.reduced_cost(u, edge_index)
                    {
                        dist[to] = Some(d + self.rn.reduced_cost(u, edge_index));
                        bh.push((-dist[to].unwrap(), to));
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

        visited[sink]
    }

    fn primal(&mut self, source: usize, sink: usize) {
        assert!(self.rn.excesses[source] > F::zero() && self.rn.excesses[sink] < F::zero());

        let mut flow = F::zero();
        while self.rn.excesses[source] > F::zero() {
            self.update_distances(source, sink);

            // no s-t path
            if self.distances[source] >= self.rn.num_nodes {
                break;
            }

            self.current_edge
                .iter_mut()
                .enumerate()
                .for_each(|(u, e)| *e = self.rn.start[u]);
            match self.dfs(source, sink, self.rn.excesses[source]) {
                Some(delta) => flow += delta,
                None => break,
            }
        }
        self.rn.excesses[source] -= flow;
        self.rn.excesses[sink] += flow;
    }

    // O(n + m)
    // calculate the distance from u to sink in the residual network
    // if such a path does not exist, distance[u] becomes self.num_nodes
    pub fn update_distances(&mut self, source: usize, sink: usize) {
        self.que.clear();
        self.que.push_back(sink);
        self.distances.fill(self.rn.num_nodes);
        self.distances[sink] = 0;

        while let Some(v) = self.que.pop_front() {
            for arc_id in self.rn.neighbors(v) {
                // e.to -> v
                let to = self.rn.to[arc_id.index()];
                if self.rn.flow[arc_id.index()] > F::zero()
                    && self.distances[to] == self.rn.num_nodes
                    && self.rn.reduced_cost_rev(v, arc_id) == F::zero()
                {
                    self.distances[to] = self.distances[v] + 1;
                    if to != source {
                        self.que.push_back(to);
                    }
                }
            }
        }
    }

    fn dfs(&mut self, u: usize, sink: usize, upper: F) -> Option<F> {
        if u == sink {
            return Some(upper);
        }

        let mut res = F::zero();
        for arc_id in self.current_edge[u]..self.rn.start[u + 1] {
            let arc_id = ArcId(arc_id);
            self.current_edge[u] = arc_id.index();

            if !self.is_admissible_edge(u, arc_id) || self.rn.reduced_cost(u, arc_id) != F::zero() {
                continue;
            }

            let v = self.rn.to[arc_id.index()];
            let residual_capacity = self.rn.residual_capacity(arc_id);
            if let Some(d) = self.dfs(v, sink, residual_capacity.min(upper - res)) {
                let rev = self.rn.rev[arc_id.index()];

                // update flow
                self.rn.flow[arc_id.index()] += d;
                self.rn.flow[rev] -= d;

                res += d;
                if res == upper {
                    return Some(res);
                }
            }
        }
        self.current_edge[u] = self.rn.start[u + 1];
        self.distances[u] = self.rn.num_nodes;

        Some(res)
    }

    #[inline]
    pub fn is_admissible_edge(&self, from: usize, arc_id: ArcId) -> bool {
        self.rn.residual_capacity(arc_id) > F::zero()
            && self.distances[from] == self.distances[self.rn.to[arc_id.index()]] + 1
    }
}
