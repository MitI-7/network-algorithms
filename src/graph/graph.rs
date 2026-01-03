use crate::graph::{
    direction::{Directed, Direction, Undirected},
    edge::Edge,
    ids::{EdgeId, NodeId},
    node::Node,
};
use std::marker::PhantomData;

#[derive(Clone, Debug)]
pub struct Graph<D: Direction, N, E> {
    nodes: Vec<Node<N>>,
    edges: Vec<Edge<E>>,
    _direction: PhantomData<D>,
}

impl<D: Direction, N, E> Graph<D, N, E> {
    pub fn num_nodes(&self) -> usize {
        self.nodes.len()
    }

    pub fn num_edges(&self) -> usize {
        self.edges.len()
    }

    pub fn add_node_with(&mut self, data: N) -> NodeId {
        let node_id = NodeId(self.num_nodes());
        self.nodes.push(Node { data });
        node_id
    }

    pub fn add_nodes_with<I>(&mut self, datas: I) -> Vec<NodeId>
    where
        I: IntoIterator<Item = N>,
    {
        datas.into_iter().map(|d| self.add_node_with(d)).collect()
    }

    pub fn add_edge(&mut self, u: NodeId, v: NodeId, data: E) -> Option<EdgeId> {
        if u.index() >= self.num_nodes() || v.index() >= self.num_nodes() {
            return None;
        }
        let edge_id = EdgeId(self.num_edges());
        self.edges.push(Edge { u, v, data });
        Some(edge_id)
    }

    pub fn get_node(&self, node_id: NodeId) -> Option<&Node<N>> {
        if node_id.index() >= self.num_nodes() {
            return None;
        }
        Some(&self.nodes[node_id.index()])
    }

    pub fn get_node_mut(&mut self, node_id: NodeId) -> Option<&mut Node<N>> {
        if node_id.index() >= self.num_nodes() {
            return None;
        }
        Some(&mut self.nodes[node_id.index()])
    }

    pub fn get_edge(&self, edge_id: EdgeId) -> Option<&Edge<E>> {
        if edge_id.index() >= self.num_edges() {
            return None;
        }
        Some(&self.edges[edge_id.index()])
    }

    pub fn edges(&self) -> std::slice::Iter<'_, Edge<E>> {
        self.edges.iter()
    }
}

impl<D: Direction, N: Default, E> Graph<D, N, E> {
    pub fn add_node(&mut self) -> NodeId {
        self.add_node_with(N::default())
    }

    pub fn add_nodes(&mut self, n: usize) -> Vec<NodeId> {
        (0..n).map(|_| self.add_node()).collect()
    }
}

impl<D: Direction, N, E> Default for Graph<D, N, E> {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            _direction: PhantomData,
        }
    }
}

impl<N, E> Graph<Directed, N, E> {
    pub fn new_directed() -> Self {
        Self::default()
    }
}

impl<N, E> Graph<Undirected, N, E> {
    pub fn new_undirected() -> Self {
        Self::default()
    }
}
