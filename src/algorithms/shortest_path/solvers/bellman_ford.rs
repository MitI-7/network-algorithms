use crate::data_structures::BitVector;
use crate::graph::edge::Edge;
use crate::ids::EdgeId;
use crate::{
    algorithms::shortest_path::{
        edge::WeightEdge,
        internal_graph::InternalGraph,
        solvers::{macros::impl_shortest_path_solver, solver::ShortestPathSolver},
        status::Status,
    },
    core::numeric::FlowNum,
    graph::{direction::Directed, graph::Graph, ids::NodeId},
};

pub struct BellmanFord<W> {
    ig: InternalGraph<W>,
    reached: BitVector,
    distances: Box<[W]>,
}

impl<W> BellmanFord<W>
where
    W: FlowNum,
{
    pub fn new(graph: &Graph<Directed, (), WeightEdge<W>>) -> Self {
        let ig = InternalGraph::from(graph, |e| e.data.weight);
        Self::new_with_internal_graph(ig)
    }

    pub fn new_graph_with<N, E, WF>(graph: &Graph<Directed, N, E>, weight_fn: WF) -> Self
    where
        WF: Fn(&Edge<E>) -> W,
    {
        let ig = InternalGraph::from(graph, weight_fn);
        Self::new_with_internal_graph(ig)
    }

    fn new_with_internal_graph(ig: InternalGraph<W>) -> Self {
        let num_nodes = ig.num_nodes;
        Self { ig, reached: BitVector::new(num_nodes), distances: vec![W::max_value(); num_nodes].into_boxed_slice() }
    }


    fn run(&mut self, source: NodeId) -> Result<(), Status> {
        self.reached.clear();
        self.distances.fill(W::max_value());
        self.distances[source.index()] = W::zero();

        let mut num_loop = 0;
        for _ in 0..self.ig.num_nodes {
            let mut update = false;
            for u in (0..self.ig.num_nodes).map(NodeId) {
                if self.distances[u.index()] == W::max_value() {
                    continue;
                }

                for edge_id in self.ig.neighbors(u).map(EdgeId) {
                    let to = self.ig.to[edge_id.index()];
                    let w = self.ig.weight[edge_id.index()];
                    let new_dist = self.distances[u.index()] + w;
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

        if num_loop == self.ig.num_nodes {
            Err(Status::NegativeCycle)
        } else {
            Ok(())
        }
    }
}

impl_shortest_path_solver!(BellmanFord, run);
