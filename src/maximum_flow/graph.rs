use num_traits::Zero;
use std::fmt::Debug;

#[derive(PartialEq, Clone, Debug)]
pub struct Edge<Flow> {
    pub from: usize,
    pub to: usize,
    pub flow: Flow,
    pub upper: Flow,
}

#[derive(Default)]
pub struct Graph<Flow> {
    num_nodes: usize,
    num_edges: usize,
    pub(crate) edges: Vec<Edge<Flow>>,
}

impl<Flow> Graph<Flow>
where
    Flow: Ord + Copy + Zero,
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
    pub fn add_directed_edge(&mut self, from: usize, to: usize, upper: Flow) -> Option<usize> {
        if from >= self.num_nodes || to >= self.num_nodes {
            return None;
        }

        self.edges.push(Edge { from, to, flow: Flow::zero(), upper });

        self.num_edges += 1;
        Some(self.num_edges - 1)
    }

    pub fn get_edge(&self, edge_id: usize) -> Option<Edge<Flow>> {
        if edge_id >= self.edges.len() {
            return None;
        }
        let edge = &self.edges[edge_id];
        Some(Edge { from: edge.from, to: edge.to, flow: edge.flow, upper: edge.upper })
    }

    pub fn pop_edge(&mut self) {
        self.edges.pop();
        self.num_edges -= 1;
    }

    pub fn reset(&mut self) {
        self.edges.iter_mut().for_each(|edge| edge.flow = Zero::zero());
    }
}
