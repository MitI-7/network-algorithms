use crate::graph::{
    direction::{Directed, Direction, Undirected},
    edge::Edge,
    ids::{EdgeId, NodeId},
    node::Node,
};
use std::marker::PhantomData;


#[derive(Clone, Debug)]
pub struct BipartiteGraph<D, N = (), E = ()> {
    left_nodes: Vec<N>,
    right_nodes: Vec<N>,
    num_edges: usize,
    pub(crate) edges: Vec<Edge<E>>,
    pub(crate) degree_left: Vec<usize>,
    pub(crate) degree_right: Vec<usize>,
    _direction: PhantomData<D>,
}

impl<D: Direction, N: Default, E> BipartiteGraph<D, N, E> {
    #[inline]
    pub fn num_left_nodes(&self) -> usize {
        self.left_nodes.len()
    }

    pub fn num_right_nodes(&self) -> usize {
        self.right_nodes.len()
    }

    #[inline]
    pub fn num_edges(&self) -> usize {
        self.num_edges
    }

    pub fn add_left_node(&mut self) -> NodeId {
        self.left_nodes.push(N::default());
        self.degree_left.push(0);
        NodeId(self.left_nodes.len() - 1)
    }

    pub fn add_left_nodes(&mut self, n: usize) -> Vec<NodeId> {
        (0..n).map(|_| self.add_left_node()).collect()
    }

    pub fn add_right_node(&mut self) -> NodeId {
        self.right_nodes.push(N::default());
        self.degree_right.push(0);
        NodeId(self.right_nodes.len() - 1)
    }

    pub fn add_right_nodes(&mut self, n: usize) -> Vec<NodeId> {
        (0..n).map(|_| self.add_right_node()).collect()
    }

    // TODO: r -> lの有向辺をどうするか
    // いまは無向辺しか表現できない
    pub fn add_directed_edge(&mut self, from: NodeId, to: NodeId, data: E) -> EdgeId {
        self.edges.push(Edge { u: from, v: to, data });
        self.degree_left[from.index()] += 1;
        self.degree_right[to.index()] += 1;
        EdgeId(self.edges.len() - 1)
    }

    pub fn add_edge(&mut self, u: NodeId, v: NodeId, data: E) -> EdgeId {
        self.edges.push(Edge { u, v, data });
        self.degree_left[u.index()] += 1;
        self.degree_right[v.index()] += 1;
        EdgeId(self.edges.len() - 1)
    }

    pub fn get_edge(&self, edge_id: usize) -> Option<&Edge<E>> {
        if edge_id >= self.edges.len() {
            return None;
        }
        Some(&self.edges[edge_id])
    }
}

impl<D: Direction, N, E> Default for BipartiteGraph<D, N, E> {
    fn default() -> Self {
        Self {
            left_nodes: Vec::new(),
            right_nodes: Vec::new(),
            num_edges: 0,
            edges: Vec::new(),
            degree_left: Vec::new(),
            degree_right: Vec::new(),
            _direction: PhantomData,
        }
    }
}

impl<N, E> BipartiteGraph<Directed, N, E> {
    pub fn new_directed() -> Self {
        Self::default()
    }
}

impl<N, E> BipartiteGraph<Undirected, N, E> {
    pub fn new_undirected() -> Self {
        Self::default()
    }
}
