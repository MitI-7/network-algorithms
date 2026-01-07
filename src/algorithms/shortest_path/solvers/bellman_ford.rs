use crate::ids::EdgeId;
use crate::{
    algorithms::shortest_path::{
        csr::CSR,
        edge::WeightEdge,
        result::ShortestPathResult,
        solvers::{macros::impl_shortest_path_solver, solver::ShortestPathSolver},
        status::Status,
    },
    core::numeric::FlowNum,
    graph::{direction::Directed, graph::Graph, ids::NodeId},
};

pub struct BellmanFord<W> {
    csr: CSR<W>,
}

impl<W> BellmanFord<W>
where
    W: FlowNum,
{
    pub fn new(graph: &Graph<Directed, (), WeightEdge<W>>) -> Self {
        let csr = CSR::new(graph);
        Self { csr }
    }

    fn run(&mut self, source: NodeId) -> Result<ShortestPathResult<W>, Status> {
        let mut distances = vec![W::max_value(); self.csr.num_nodes];
        distances[source.index()] = W::zero();

        let mut num_loop = 0;
        for _ in 0..self.csr.num_nodes {
            let mut update = false;
            for u in (0..self.csr.num_nodes).map(NodeId) {
                if distances[u.index()] == W::max_value() {
                    continue;
                }

                for edge_id in self.csr.neighbors(u).map(EdgeId) {
                    let to = self.csr.to[edge_id.index()];
                    let w = self.csr.weight[edge_id.index()];
                    let new_dist = distances[to.index()] + w;
                    if new_dist < distances[to.index()] {
                        distances[to.index()] = distances[u.index()] + w;
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
            Ok(ShortestPathResult { distances })
        }
    }
}

impl_shortest_path_solver!(BellmanFord, run);
