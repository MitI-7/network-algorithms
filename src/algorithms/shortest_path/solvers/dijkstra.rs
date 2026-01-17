use crate::{
    algorithms::shortest_path::{
        edge::WeightEdge,
        internal_graph::InternalGraph,
        solvers::{macros::impl_shortest_path_solver, solver::ShortestPathSolver},
        status::Status,
    },
    core::numeric::FlowNum,
    data_structures::BitVector,
    graph::{
        direction::Directed,
        edge::Edge,
        graph::Graph,
        ids::{EdgeId, INVALID_NODE_ID, NodeId},
    },
};
use std::{cmp::Reverse, collections::BinaryHeap};

pub struct Dijkstra<W> {
    csr: InternalGraph<W>,
    reached: BitVector,
    distances: Box<[W]>,
}

impl<W> Dijkstra<W>
where
    W: FlowNum,
{
    pub fn new(graph: &Graph<Directed, (), WeightEdge<W>>) -> Self {
        let csr = InternalGraph::from(graph, |e| e.data.weight);
        let num_nodes = csr.num_nodes;
        Self { csr, reached: BitVector::new(num_nodes), distances: vec![W::max_value(); num_nodes].into_boxed_slice() }
    }

    pub fn new_graph_with<N, E, WF>(graph: &Graph<Directed, N, E>, weight_fn: WF) -> Self
    where
        WF: Fn(&Edge<E>) -> W,
    {
        let csr = InternalGraph::from(graph, weight_fn);
        let num_nodes = csr.num_nodes;
        Self { csr, reached: BitVector::new(num_nodes), distances: vec![W::max_value(); num_nodes].into_boxed_slice() }
    }

    fn run(&mut self, source: NodeId) -> Result<(), Status> {
        if self.csr.weight.iter().any(|&w| w < W::zero()) {
            return Err(Status::BadInput);
        }

        let mut heap = BinaryHeap::new();
        heap.push((Reverse(W::zero()), source));

        let mut prev = vec![INVALID_NODE_ID; self.csr.num_nodes];
        self.reached.clear();
        self.distances.fill(W::max_value());
        self.distances[source.index()] = W::zero();

        while let Some((d, u)) = heap.pop() {
            if self.reached.get(u.index()) {
                continue;
            }
            self.reached.set(u.index(), true);

            for edge_id in self.csr.neighbors(u).map(EdgeId) {
                let to = self.csr.to[edge_id.index()];
                let w = self.csr.weight[edge_id.index()];

                if self.reached.get(to.index()) {
                    continue;
                }

                let new_dist = d.0 + w;
                if new_dist < self.distances[to.index()] {
                    self.distances[to.index()] = new_dist;
                    prev[to.index()] = u;
                    heap.push((Reverse(new_dist), to));
                }
            }
        }
        Ok(())
    }
}

impl_shortest_path_solver!(Dijkstra, run);
