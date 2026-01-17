use crate::data_structures::BitVector;
use crate::ids::EdgeId;
use crate::{
    algorithms::shortest_path::{
        internal_graph::InternalGraph,
        edge::WeightEdge,
        solvers::{macros::impl_shortest_path_solver, solver::ShortestPathSolver},
        status::Status,
    },
    core::numeric::FlowNum,
    graph::{direction::Directed, graph::Graph, ids::NodeId},
};

pub struct BellmanFord<W> {
    csr: InternalGraph<W>,
    reached: BitVector,
    distances: Box<[W]>,
}

impl<W> BellmanFord<W>
where
    W: FlowNum,
{
    pub fn new(graph: &Graph<Directed, (), WeightEdge<W>>) -> Self {
        let csr = InternalGraph::new(graph);
        let num_nodes = csr.num_nodes;
        Self { csr, reached: BitVector::new(num_nodes), distances: vec![W::max_value(); num_nodes].into_boxed_slice() }
    }

    fn run(&mut self, source: NodeId) -> Result<(), Status> {
        self.reached.clear();
        self.distances[source.index()] = W::zero();

        let mut num_loop = 0;
        for _ in 0..self.csr.num_nodes {
            let mut update = false;
            for u in (0..self.csr.num_nodes).map(NodeId) {
                if self.distances[u.index()] == W::max_value() {
                    continue;
                }

                for edge_id in self.csr.neighbors(u).map(EdgeId) {
                    let to = self.csr.to[edge_id.index()];
                    let w = self.csr.weight[edge_id.index()];
                    let new_dist = self.distances[to.index()] + w;
                    if new_dist < self.distances[to.index()] {
                        self.distances[to.index()] = self.distances[u.index()] + w;
                        self.reached.set(to.index(), true);
                        update = true;
                    }
                }
            }
            if !update {
                break;
            }
            num_loop += 1;
        }

        if num_loop == self.csr.num_nodes {
            Err(Status::NegativeCycle)
        } else {
            Ok(())
        }
    }
}

impl_shortest_path_solver!(BellmanFord, run);
