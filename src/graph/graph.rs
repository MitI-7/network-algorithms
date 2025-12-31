use crate::graph::{
    direction::{Directed, Direction, Undirected},
    edge::Edge,
    ids::{EdgeId, NodeId},
    node::Node,
};
use std::marker::PhantomData;

#[derive(Clone, Debug)]
pub struct Graph<D: Direction, N, E> {
    pub(crate) nodes: Vec<Node<N>>,
    pub(crate) edges: Vec<Edge<E>>,
    _direction: PhantomData<D>,
}

impl<D: Direction, N: Default, E> Graph<D, N, E> {
    pub fn num_nodes(&self) -> usize {
        self.nodes.len()
    }

    pub fn num_edges(&self) -> usize {
        self.edges.len()
    }

    pub fn add_node(&mut self) -> NodeId {
        let node_id = NodeId(self.num_nodes());
        self.nodes.push(Node {u: node_id, data: N::default()});
        node_id
    }

    pub fn add_nodes(&mut self, n: usize) -> Vec<NodeId> {
        // let start = self.num_nodes();
        // self.nodes.extend(n);
        // (start..start + n).map(NodeId).collect()
        (0..n).map(|_| self.add_node()).collect()
    }

    pub fn add_edge(&mut self, u: NodeId, v: NodeId, data: E) -> EdgeId {
        self.edges.push(Edge { u, v, data });
        EdgeId(self.num_edges() - 1)
    }
    
    pub fn get_node_mut(&mut self, node_id: NodeId) -> &mut Node<N> {
        &mut self.nodes[node_id.index()]
    }

    pub fn get_edge(&self, edge_id: EdgeId) -> &Edge<E> {
        &self.edges[edge_id.index()]
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
