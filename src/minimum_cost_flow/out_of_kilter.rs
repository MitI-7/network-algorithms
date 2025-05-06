use crate::minimum_cost_flow::csr::CSR;
use crate::minimum_cost_flow::graph::Graph;
use crate::minimum_cost_flow::status::Status;
use crate::minimum_cost_flow::{MinimumCostFlowNum, MinimumCostFlowSolver};
use std::cmp::Reverse;
use std::collections::BinaryHeap;

// O(nU * (m + n) log n)
#[derive(Default)]
pub struct OutOfKilter<Flow> {
    csr: CSR<Flow>,
}

impl<Flow> MinimumCostFlowSolver<Flow> for OutOfKilter<Flow>
where
    Flow: MinimumCostFlowNum,
{
    fn solve(&mut self, graph: &mut Graph<Flow>) -> Result<Flow, Status> {
        if graph.is_unbalance() {
            return Err(Status::Unbalanced);
        }

        let (_source, artificial_nodes, artificial_edges) = graph.construct_extend_network_feasible_solution();
        self.csr.build(graph);

        let mut out_of_kilter_edges = Vec::new();
        for edge_id in 0..self.csr.to.len() {
            let rev = self.csr.rev[edge_id];
            let p = self.csr.to[rev];
            if self.kilter_number(p, edge_id) != Flow::zero() {
                let q = self.csr.to[edge_id];
                out_of_kilter_edges.push((p, q, edge_id));
            }
        }

        'outer: for (p, q, edge_id) in out_of_kilter_edges {
            while self.kilter_number(p, edge_id) > Flow::zero() {
                let (dist, prev) = self.shortest_path(q);
                if prev[p].is_none() {
                    break 'outer;
                }

                // update potentials
                for u in 0..self.csr.num_nodes {
                    if let Some(d) = dist[u] {
                        self.csr.potentials[u] -= d;
                    }
                }

                // update flow
                if self.csr.reduced_cost(p, edge_id) < Flow::zero() {
                    self.update_flow_in_cycle(q, edge_id, prev);
                }
            }
        }

        self.csr.set_flow(graph);

        let status = if artificial_edges.iter().all(|&edge_id| graph.edges[edge_id].flow == Flow::zero()) {
            Status::Optimal
        } else {
            Status::Infeasible
        };
        graph.remove_artificial_sub_graph(&artificial_nodes, &artificial_edges);

        if status == Status::Optimal {
            Ok(graph.minimum_cost())
        } else {
            Err(status)
        }
    }
}

impl<Flow> OutOfKilter<Flow>
where
    Flow: MinimumCostFlowNum,
{
    pub fn solve(&mut self, graph: &mut Graph<Flow>) -> Result<Flow, Status> {
        <Self as MinimumCostFlowSolver<Flow>>::solve(self, graph)
    }

    fn kilter_number(&self, u: usize, edge_id: usize) -> Flow {
        if self.csr.reduced_cost(u, edge_id) >= Flow::zero() {
            Flow::zero()
        } else {
            self.csr.residual_capacity(edge_id)
        }
    }

    fn shortest_path(&mut self, s: usize) -> (Vec<Option<Flow>>, Vec<Option<usize>>) {
        let mut prev = vec![None; self.csr.num_nodes];
        let mut bh = BinaryHeap::new();
        let mut dist: Vec<Option<Flow>> = vec![None; self.csr.num_nodes];
        let mut visited = vec![false; self.csr.num_nodes];

        bh.push((Reverse(Flow::zero()), s));
        dist[s] = Some(Flow::zero());

        while let Some((d, u)) = bh.pop() {
            if visited[u] {
                continue;
            }
            visited[u] = true;

            for edge_id in self.csr.start[u]..self.csr.start[u + 1] {
                if self.csr.residual_capacity(edge_id) <= Flow::zero() {
                    continue;
                }

                let to = self.csr.to[edge_id];
                let new_dist = d.0 + self.csr.reduced_cost(u, edge_id).max(Flow::zero());
                if dist[to].is_none() || dist[to].unwrap() > new_dist {
                    dist[to] = Some(new_dist);
                    prev[to] = Some(edge_id);
                    bh.push((Reverse(new_dist), to));
                }
            }
        }

        (dist, prev)
    }

    fn update_flow_in_cycle(&mut self, q: usize, edge_id: usize, mut prev: Vec<Option<usize>>) {
        prev[q] = Some(edge_id); // p -> q

        // calculate delta
        let mut delta = self.csr.residual_capacity(edge_id);
        let mut v = q;
        while let Some(edge_idx) = prev[v] {
            delta = delta.min(self.csr.residual_capacity(edge_idx));
            let rev = self.csr.rev[edge_idx];
            v = self.csr.to[rev];
            if v == q {
                break;
            }
        }

        // update flow
        let mut v = q;
        while let Some(edge_id) = prev[v] {
            let rev = self.csr.rev[edge_id];
            v = self.csr.to[rev];
            self.csr.push_flow(v, edge_id, delta);
            if v == q {
                break;
            }
        }
    }
}
