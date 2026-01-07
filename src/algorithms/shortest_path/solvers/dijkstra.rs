use crate::{
    algorithms::shortest_path::{
        csr::CSR,
        edge::WeightEdge,
        result::ShortestPathResult,
        solvers::{macros::impl_shortest_path_solver, solver::ShortestPathSolver},
        status::Status,
    },
    core::numeric::FlowNum,
    data_structures::bit_vector,
    graph::{
        direction::Directed,
        graph::Graph,
        ids::{EdgeId, INVALID_NODE_ID, NodeId},
    },
};
use std::{cmp::Reverse, collections::BinaryHeap};

#[derive(Default)]
pub struct Dijkstra<W> {
    csr: CSR<W>,
}

impl<W> Dijkstra<W>
where
    W: FlowNum,
{
    pub fn new(graph: &Graph<Directed, (), WeightEdge<W>>) -> Self {
        let csr = CSR::new(graph);
        Self { csr }
    }

    fn run(&mut self, source: NodeId) -> Result<ShortestPathResult<W>, Status> {
        let mut heap = BinaryHeap::new();
        heap.push((Reverse(W::zero()), source));

        let mut visited = bit_vector::BitVector::new(self.csr.num_nodes);
        let mut distances = vec![W::max_value(); self.csr.num_nodes];
        let mut prev = vec![INVALID_NODE_ID; self.csr.num_nodes];
        distances[source.index()] = W::zero();

        while let Some((d, u)) = heap.pop() {
            if visited.get(u.index()) {
                continue;
            }
            visited.set(u.index(), true);

            for edge_id in self.csr.neighbors(u).map(EdgeId) {
                let to = self.csr.to[edge_id.index()];
                let w = self.csr.weight[edge_id.index()];

                if visited.get(to.index()) {
                    continue;
                }

                let new_dist = d.0 + w;
                if new_dist < distances[to.index()] {
                    distances[to.index()] = new_dist;
                    prev[to.index()] = u;
                    heap.push((Reverse(new_dist), to));
                }
            }
        }
        Ok(ShortestPathResult { distances })
    }
}

impl_shortest_path_solver!(Dijkstra, run);
