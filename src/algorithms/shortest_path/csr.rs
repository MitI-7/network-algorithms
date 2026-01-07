use crate::core::graph::Graph;
use crate::core::direction::Directed;
use crate::edge::weight::WeightEdge;
use crate::traits::*;

#[derive(Default)]
pub struct CSR<W> {
    pub num_nodes: usize,
    pub num_edges: usize,

    pub start: Box<[usize]>,
    pub to: Box<[usize]>,
    pub weight: Box<[W]>,
}

impl<W> CSR<W>
where
    W: Ord +  Zero + Clone + Copy,
{
    pub fn build(&mut self, graph: &Graph<Directed, (), WeightEdge<W>>) {
        self.num_nodes = graph.num_nodes();
        self.num_edges = graph.num_edges();

        // initialize
        self.start = vec![0; self.num_nodes + 1].into_boxed_slice();
        self.to = vec![usize::MAX; self.num_edges].into_boxed_slice();
        self.weight = vec![W::zero(); self.num_edges].into_boxed_slice();

        let mut degree = vec![0; self.num_nodes].into_boxed_slice();
        for edge in graph.edges.iter() {
            degree[edge.u.index()] += 1;
        }

        for u in 1..=self.num_nodes {
            self.start[u] += self.start[u - 1] + degree[u - 1];
        }

        let mut counter = vec![0; self.num_nodes];
        for edge in graph.edges.iter() {
            let (u, v) = (edge.u.index(), edge.v.index());
            self.to[self.start[u] + counter[u]] = v;
            self.weight[self.start[u] + counter[u]] = edge.data.weight;

            counter[u] += 1;
        }
    }

    #[inline]
    pub fn neighbors(&self, u: usize) -> std::ops::Range<usize> {
        self.start[u]..self.start[u + 1]
    }
}
