use crate::core::direction::{Directed, Direction, Undirected};
use crate::core::ids::{EdgeId, NodeId};
use std::marker::PhantomData;

#[derive(Clone, Debug)]
pub struct Edge<E> {
    pub u: NodeId,
    pub v: NodeId,
    pub data: E,
}

#[derive(Clone, Debug)]
pub struct Graph<D: Direction, N = (), E = ()> {
    pub nodes: Vec<N>,
    pub edges: Vec<Edge<E>>,
    _direction: PhantomData<D>,
}

impl<D: Direction, N: Default, E> Graph<D, N, E> {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            _direction: PhantomData,
        }
    }

    pub fn add_node(&mut self) -> NodeId {
        let id = NodeId(self.nodes.len());
        self.nodes.push(N::default());
        id
    }

    pub fn add_nodes(&mut self, n: usize) -> Vec<NodeId> {
        (0..n).map(|_| self.add_node()).collect()
    }
    
    pub fn pop_node(&mut self) {
        self.nodes.pop();
    }

    pub fn add_directed_edge(&mut self, from: NodeId, to: NodeId, data: E) -> EdgeId {
        self.edges.push(Edge { u: from, v: to, data });
        EdgeId(self.edges.len() - 1)
    }

    pub fn add_edge(&mut self, u: NodeId, v: NodeId, data: E) -> EdgeId {
        self.edges.push(Edge { u, v, data });
        EdgeId(self.edges.len() - 1)
    }

    pub fn get_edge(&self, edge_id: EdgeId) -> &Edge<E> {
        &self.edges[edge_id.index()]
    }
    
    pub fn pop_edge(&mut self) {
        self.edges.pop();   
    }

    // pub fn add_node_value(&mut self, v: NodeId, val: N)
    // where
    //     N: AddAssign,
    // {
    //     if v.0 >= self.nodes.len() {
    //         self.nodes.resize(v.0 + 1, N::default());
    //     }
    //     self.nodes[v.0] += val;
    // }

    pub fn edges(&self) -> &[Edge<E>] {
        &self.edges
    }
    
    pub fn node_payload(&self) -> &[N] {
        &self.nodes
    }
    
    pub fn num_nodes(&self) -> usize {
        self.nodes.len()
    }
    
    pub fn num_edges(&self) -> usize {
        self.edges.len()
    }
}

impl<D: Direction, N, E> Default for Graph<D, N, E> {
    fn default() -> Self {
        Self { nodes: Vec::new(), edges: Vec::new(), _direction: PhantomData }
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

