use crate::minimum_cost_flow::csr::{CSR, construct_extend_network_one_supply_one_demand};
use crate::graph::graph::{CapCostEdge, Directed, Graph, ExcessNode, EdgeId};
use crate::minimum_cost_flow::status::Status;
use crate::minimum_cost_flow::{MinimumCostFlowNum, MinimumCostFlowSolver};
use std::collections::{BinaryHeap, VecDeque};
use crate::minimum_cost_flow::translater::translater;

#[derive(Default)]
pub struct PrimalDual<Flow> {
    csr: CSR<Flow>,

    // maximum flow(dinic)
    que: VecDeque<usize>,
    distances: Vec<usize>,
    current_edge: Vec<usize>,
}

impl<Flow> MinimumCostFlowSolver<Flow> for PrimalDual<Flow>
where
    Flow: MinimumCostFlowNum,
{
    fn solve(&mut self, graph: &mut Graph<Directed, ExcessNode<Flow>, CapCostEdge<Flow>>) -> Result<Flow, Status> {
        // if graph.is_unbalance() {
        //     return Err(Status::Unbalanced);
        // }
        
        let mut t = Flow::zero();
        for u in 0..graph.num_nodes() {
            t += graph.nodes[u].b;
        }
        if t != Flow::zero() {
            return Err(Status::Unbalanced);
        }
        
        if graph.num_nodes() == 0 {
            return Ok(Flow::zero());
        }

        if graph.num_edges() == 0 {
            for u in 0..graph.num_nodes() {
                if graph.nodes[u].b != Flow::zero() {
                    return Err(Status::Infeasible);
                }
            }
            return Ok(Flow::zero());
        }

        let mut new_graph = translater(graph);

        // transforms the minimum cost flow problem into a problem with a single excess node and a single deficit node.
        let (source, sink, artificial_edges) = construct_extend_network_one_supply_one_demand(&mut new_graph);
        self.csr.build(&new_graph, Some(&[source, sink]), Some(&artificial_edges));

        self.distances.resize(self.csr.num_nodes, 0);
        self.current_edge.resize(self.csr.num_nodes, 0);

        while self.csr.excesses[source.index()] > Flow::zero() {
            if !self.dual(source.index(), sink.index()) {
                break;
            }
            self.primal(source.index(), sink.index());
        }

        self.csr.set_flow(graph);

        // graph.remove_artificial_sub_graph(&artificial_nodes, &artificial_edges);
        if self.csr.excesses[source.index()] != Flow::zero() || self.csr.excesses[sink.index()] != Flow::zero() {
            return Err(Status::Infeasible);
        }

            Ok((0..graph.num_edges()).fold(Flow::zero(), |cost, edge_id| {
                let edge = graph.get_edge(EdgeId(edge_id));
                cost + edge.data.cost * edge.data.flow
            }))

    }
}

impl<Flow> PrimalDual<Flow>
where
    Flow: MinimumCostFlowNum,
{
    pub fn solve(&mut self, graph: &mut Graph<Directed, ExcessNode<Flow>, CapCostEdge<Flow>>) -> Result<Flow, Status> {
        <Self as MinimumCostFlowSolver<Flow>>::solve(self, graph)
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
                    if dist[to].is_none() || dist[to].unwrap() > d + self.csr.reduced_cost(u, edge_index) {
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

            self.current_edge.iter_mut().enumerate().for_each(|(u, e)| *e = self.csr.start[u]);
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
                if self.csr.flow[i] > Flow::zero() && self.distances[to] == self.csr.num_nodes && self.csr.reduced_cost_rev(v, i) == Flow::zero() {
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

            if !self.is_admissible_edge(u, edge_index) || self.csr.reduced_cost(u, edge_index) != Flow::zero() {
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
        self.csr.residual_capacity(i) > Flow::zero() && self.distances[from] == self.distances[self.csr.to[i]] + 1
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use crate::graph::graph::Graph;

    #[test]
    fn test() {
        let mut g:Graph<Directed, ExcessNode<i32>, CapCostEdge<i32>> = Graph::new();
        let a = g.add_node();
        g.add_edge(a, a, CapCostEdge{flow: 0, lower: -5, upper: 0, cost:10});

        let s = PrimalDual::default().solve(&mut g).unwrap();
        println!("{}", s);

        // 1 20 77
        // 0
        // 0 0 -1 9 0
        // 0 0 -6 0 -10
        // 0 0 8 8 0
        // 0 0 -8 -4 -4
        // 0 0 -10 0 -3
        // 0 0 -10 1 -2
        // 0 0 -8 10 8
        // 0 0 1 10 2
        // 0 0 10 10 3
        // 0 0 -5 5 1
        // 0 0 -10 -6 -6
        // 0 0 -3 7 8
        // 0 0 0 9 0
        // 0 0 0 0 -6
        // 0 0 4 8 8
        // 0 0 0 2 6
        // 0 0 -10 -3 -6
        // 0 0 -10 -8 -1
        // 0 0 -10 0 0
        // 0 0 -10 -10 -3
        //




    }
}
