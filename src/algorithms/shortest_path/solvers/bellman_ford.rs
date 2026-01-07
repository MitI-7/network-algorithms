use crate::ids::EdgeId;
use crate::{
    algorithms::shortest_path::{csr::CSR, edge::WeightEdge, result::ShortestPathResult, status::Status},
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

    pub fn solve(&mut self, source: NodeId) -> Result<ShortestPathResult<W>, Status> {
        let mut distances = vec![None; self.csr.num_nodes];
        distances[source.index()] = Some(W::zero());

        let mut num_loop = 0;
        loop {
            let mut update = false;
            for u in (0..self.csr.num_nodes).map(NodeId) {
                if distances[u.index()].is_none() {
                    continue;
                }

                for edge_id in self.csr.neighbors(u).map(EdgeId) {
                    let to = self.csr.to[edge_id.index()];
                    let w = self.csr.weight[edge_id.index()];
                    if distances[to.index()].is_none()
                        || (distances[u.index()].is_some()
                            && distances[to.index()].unwrap() > distances[u.index()].unwrap() + w)
                    {
                        distances[to.index()] = Some(distances[u.index()].unwrap() + w);
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
