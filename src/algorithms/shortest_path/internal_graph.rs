use crate::{
    algorithms::shortest_path::edge::WeightEdge,
    core::numeric::FlowNum,
    graph::{direction::Directed, graph::Graph},
    ids::{INVALID_NODE_ID, NodeId},
};

#[derive(Default)]
pub struct InternalGraph<W> {
    pub num_nodes: usize,
    pub _num_edges: usize,

    pub start: Box<[usize]>,
    pub to: Box<[NodeId]>,
    pub weight: Box<[W]>,
}

impl<W> InternalGraph<W>
where
    W: FlowNum,
{
    pub fn new(graph: &Graph<Directed, (), WeightEdge<W>>) -> Self {
        let num_nodes = graph.num_nodes();
        let num_edges = graph.num_edges();

        let mut csr = Self {
            num_nodes,
            _num_edges: num_edges,
            start: vec![0; num_nodes + 1].into_boxed_slice(),
            to: vec![INVALID_NODE_ID; num_edges].into_boxed_slice(),
            weight: vec![W::zero(); num_edges].into_boxed_slice(),
        };
        csr.build(graph);

        csr
    }

    fn build(&mut self, graph: &Graph<Directed, (), WeightEdge<W>>) {
        let mut degree = vec![0; self.num_nodes].into_boxed_slice();
        for edge in graph.edges() {
            degree[edge.u.index()] += 1;
        }

        for u in 1..=self.num_nodes {
            self.start[u] += self.start[u - 1] + degree[u - 1];
        }

        let mut counter = vec![0; self.num_nodes];
        for edge in graph.edges() {
            let (u, v) = (edge.u, edge.v);
            self.to[self.start[u.index()] + counter[u.index()]] = v;
            self.weight[self.start[u.index()] + counter[u.index()]] = edge.data.weight;

            counter[u.index()] += 1;
        }
    }

    #[inline]
    pub fn neighbors(&self, u: NodeId) -> std::ops::Range<usize> {
        self.start[u.index()]..self.start[u.index() + 1]
    }
}
