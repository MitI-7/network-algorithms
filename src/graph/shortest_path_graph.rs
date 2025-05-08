use crate::traits::Zero;
use std::fmt::Debug;

#[derive(PartialEq, Clone, Debug)]
pub struct Edge<W> {
    pub from: usize,
    pub to: usize,
    pub weight: W,
}

#[derive(Default)]
pub struct Graph<W> {
    num_nodes: usize,
    num_edges: usize,
    pub(crate) edges: Vec<Edge<W>>,
}

impl<W> Graph<W>
where
    W: Ord + Copy + Zero + Clone,
{
    #[inline]
    pub fn num_nodes(&self) -> usize {
        self.num_nodes
    }

    #[inline]
    pub fn num_edges(&self) -> usize {
        self.num_edges
    }

    pub fn add_node(&mut self) -> usize {
        self.num_nodes += 1;
        self.num_nodes - 1
    }

    pub fn add_nodes(&mut self, num_nodes: usize) -> Vec<usize> {
        self.num_nodes += num_nodes;
        ((self.num_nodes - num_nodes)..self.num_nodes).collect()
    }

    pub fn pop_node(&mut self) {
        self.num_nodes -= 1;
    }

    // return edge index
    pub fn add_directed_edge(&mut self, from: usize, to: usize, weight: W) -> Option<usize> {
        if from >= self.num_nodes || to >= self.num_nodes {
            return None;
        }

        self.edges.push(Edge { from, to, weight});

        self.num_edges += 1;
        Some(self.num_edges - 1)
    }

    pub fn get_edge(&self, edge_id: usize) -> Option<Edge<W>> {
        if edge_id >= self.edges.len() {
            return None;
        }
        let edge = &self.edges[edge_id];
        Some(Edge { from: edge.from, to: edge.to, weight: edge.weight})
    }

    pub fn pop_edge(&mut self) {
        self.edges.pop();
        self.num_edges -= 1;
    }
}
