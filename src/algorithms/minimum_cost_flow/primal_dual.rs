use crate::algorithms::minimum_cost_flow::edge::MinimumCostFlowEdge;
use crate::algorithms::minimum_cost_flow::node::MinimumCostFlowNode;
use crate::algorithms::minimum_cost_flow::normalized_network::NormalizedNetwork;
use crate::algorithms::minimum_cost_flow::residual_network::{ResidualNetwork, construct_extend_network_one_supply_one_demand};
use crate::algorithms::minimum_cost_flow::result::MinimumCostFlowResult;
use crate::algorithms::minimum_cost_flow::status::Status;
use crate::algorithms::minimum_cost_flow::{MinimumCostFlowNum, MinimumCostFlowSolver};
use crate::graph::direction::Directed;
use crate::graph::graph::Graph;
use crate::graph::ids::{EdgeId, NodeId};
use std::collections::{BinaryHeap, VecDeque};

#[derive(Default)]
pub struct PrimalDual<F> {
    csr: ResidualNetwork<F>,

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
        graph: &mut Graph<Directed, MinimumCostFlowNode<F>, MinimumCostFlowEdge<F>>,
    ) -> Result<MinimumCostFlowResult<F>, Status> {
        self.run(graph)
    }
}

impl<Flow> PrimalDual<Flow>
where
    Flow: MinimumCostFlowNum,
{
    pub fn run(
        &mut self,
        graph: &mut Graph<Directed, MinimumCostFlowNode<Flow>, MinimumCostFlowEdge<Flow>>,
    ) -> Result<MinimumCostFlowResult<Flow>, Status> {
        // if (0..graph.num_nodes()).into_iter().fold(Flow::zero(), |sum, u| sum + graph.nodes[u].data.b) != Flow::zero() {
        //     return Err(Status::Unbalanced);
        // }

        let mut t = Flow::zero();
        for u in 0..graph.num_nodes() {
            t += graph.get_node(NodeId(u)).data.b;
        }
        if t != Flow::zero() {
            return Err(Status::Unbalanced);
        }

        if graph.num_nodes() == 0 {
            return Ok(MinimumCostFlowResult {
                objective_value: Flow::zero(),
                flows: vec![Flow::zero(); graph.num_edges()],
            });
        }

        if graph.num_edges() == 0 {
            for u in 0..graph.num_nodes() {
                if graph.get_node(NodeId(u)).data.b != Flow::zero() {
                    return Err(Status::Infeasible);
                }
            }
            return Ok(MinimumCostFlowResult {
                objective_value: Flow::zero(),
                flows: vec![Flow::zero(); graph.num_edges()],
            });
        }

        let nn = NormalizedNetwork::new(graph);

        // transforms the minimum cost flow problem into a problem with a single excess node and a single deficit node.
        let (source, sink, artificial_edges, excess_fix) =
            construct_extend_network_one_supply_one_demand(&nn);
        self.csr
            .build(&nn, Some(&[source, sink]), Some(&artificial_edges), Some(&excess_fix));

        self.distances.resize(self.csr.num_nodes, 0);
        self.current_edge.resize(self.csr.num_nodes, 0);

        while self.csr.excesses[source.index()] > Flow::zero() {
            if !self.dual(source.index(), sink.index()) {
                break;
            }
            self.primal(source.index(), sink.index());
        }

        let flows = self.csr.get_flow(graph);

        // graph.remove_artificial_sub_graph(&artificial_nodes, &artificial_edges);
        if self.csr.excesses[source.index()] != Flow::zero()
            || self.csr.excesses[sink.index()] != Flow::zero()
        {
            return Err(Status::Infeasible);
        }

        Ok(MinimumCostFlowResult {
            objective_value: (0..graph.num_edges()).fold(Flow::zero(), |cost, edge_id| {
                let edge = graph.get_edge(EdgeId(edge_id));
                cost + edge.data.cost * flows[edge_id]
            }),
            flows,
        })
    }

    // update potentials
    fn dual(&mut self, source: usize, sink: usize) -> bool {
        assert!(self.csr.excesses[source] > Flow::zero());

        // calculate the shortest path
        let mut dist: Vec<Option<Flow>> = vec![None; self.csr.num_nodes];
        let mut visited = vec![false; self.csr.num_nodes];
        {
            let mut bh: BinaryHeap<(Flow, usize)> = BinaryHeap::new();

            bh.push((Flow::zero(), source));
            dist[source] = Some(Flow::zero());

            while let Some((mut d, u)) = bh.pop() {
                d = -d;

                if visited[u] {
                    continue;
                }
                visited[u] = true;

                for edge_index in self.csr.neighbors(u) {
                    if self.csr.residual_capacity(edge_index) == Flow::zero() {
                        continue;
                    }
                    let to = self.csr.to[edge_index];
                    if dist[to].is_none()
                        || dist[to].unwrap() > d + self.csr.reduced_cost(u, edge_index)
                    {
                        dist[to] = Some(d + self.csr.reduced_cost(u, edge_index));
                        bh.push((-dist[to].unwrap(), to));
                    }
                }
            }
        }

        // update potentials
        for u in 0..self.csr.num_nodes {
            if visited[u] {
                self.csr.potentials[u] -= dist[u].unwrap();
            }
        }

        visited[sink]
    }

    fn primal(&mut self, source: usize, sink: usize) {
        assert!(self.csr.excesses[source] > Flow::zero() && self.csr.excesses[sink] < Flow::zero());

        let mut flow = Flow::zero();
        while self.csr.excesses[source] > Flow::zero() {
            self.update_distances(source, sink);

            // no s-t path
            if self.distances[source] >= self.csr.num_nodes {
                break;
            }

            self.current_edge
                .iter_mut()
                .enumerate()
                .for_each(|(u, e)| *e = self.csr.start[u]);
            match self.dfs(source, sink, self.csr.excesses[source]) {
                Some(delta) => flow += delta,
                None => break,
            }
        }
        self.csr.excesses[source] -= flow;
        self.csr.excesses[sink] += flow;
    }

    // O(n + m)
    // calculate the distance from u to sink in the residual network
    // if such a path does not exist, distance[u] becomes self.num_nodes
    pub fn update_distances(&mut self, source: usize, sink: usize) {
        self.que.clear();
        self.que.push_back(sink);
        self.distances.fill(self.csr.num_nodes);
        self.distances[sink] = 0;

        while let Some(v) = self.que.pop_front() {
            for i in self.csr.neighbors(v) {
                // e.to -> v
                let to = self.csr.to[i];
                if self.csr.flow[i] > Flow::zero()
                    && self.distances[to] == self.csr.num_nodes
                    && self.csr.reduced_cost_rev(v, i) == Flow::zero()
                {
                    self.distances[to] = self.distances[v] + 1;
                    if to != source {
                        self.que.push_back(to);
                    }
                }
            }
        }
    }

    fn dfs(&mut self, u: usize, sink: usize, upper: Flow) -> Option<Flow> {
        if u == sink {
            return Some(upper);
        }

        let mut res = Flow::zero();
        for edge_index in self.current_edge[u]..self.csr.start[u + 1] {
            self.current_edge[u] = edge_index;

            if !self.is_admissible_edge(u, edge_index)
                || self.csr.reduced_cost(u, edge_index) != Flow::zero()
            {
                continue;
            }

            let v = self.csr.to[edge_index];
            let residual_capacity = self.csr.residual_capacity(edge_index);
            if let Some(d) = self.dfs(v, sink, residual_capacity.min(upper - res)) {
                let rev = self.csr.rev[edge_index];

                // update flow
                self.csr.flow[edge_index] += d;
                self.csr.flow[rev] -= d;

                res += d;
                if res == upper {
                    return Some(res);
                }
            }
        }
        self.current_edge[u] = self.csr.start[u + 1];
        self.distances[u] = self.csr.num_nodes;

        Some(res)
    }

    #[inline]
    pub fn is_admissible_edge(&self, from: usize, i: usize) -> bool {
        self.csr.residual_capacity(i) > Flow::zero()
            && self.distances[from] == self.distances[self.csr.to[i]] + 1
    }
}
