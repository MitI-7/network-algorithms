use crate::core::direction::Direction;
use crate::core::ids::{EdgeId, NodeId};
use std::marker::PhantomData;
use std::ops::AddAssign;

#[derive(Clone, Debug)]
pub struct Edge<E> {
    pub from: NodeId,
    pub to: NodeId,
    pub data: E,
}

#[derive(Clone, Debug)]
pub struct Graph<D: Direction, N: Default + Clone = (), E: Clone = ()> {
    pub nodes: Vec<N>,
    pub edges: Vec<Edge<E>>,
    _direction: PhantomData<D>,
}

impl<D: Direction, N: Default + Clone, E: Clone> Graph<D, N, E> {
    pub fn new() -> Self {
        Self { edges: Vec::new(), nodes: Vec::new(), _direction: PhantomData }
    }

    pub fn add_node(&mut self) -> NodeId {
        let id = NodeId(self.nodes.len());
        self.nodes.push(N::default());
        id
    }

    pub fn add_nodes(&mut self, n: usize) -> Vec<NodeId> {
        (0..n).map(|_| self.add_node()).collect()
    }

    pub fn add_directed_edge(&mut self, from: NodeId, to: NodeId, data: E) -> EdgeId {
        let eid = EdgeId(self.edges.len());
        self.edges.push(Edge { from, to, data });
        eid
    }

    pub fn add_edge(&mut self, u: NodeId, v: NodeId, data: E) -> EdgeId {
        if D::IS_DIRECTED {
            self.add_directed_edge(u, v, data)
        } else {
            let eid = self.add_directed_edge(u, v, data.clone());
            self.add_directed_edge(v, u, data); // second id unused by caller
            eid
        }
    }

    pub fn get_edge(&self, edge_id: EdgeId) -> &Edge<E> {
        &self.edges[edge_id.0]
    }

    pub fn add_node_value(&mut self, v: NodeId, val: N)
    where
        N: AddAssign,
    {
        if v.0 >= self.nodes.len() {
            self.nodes.resize(v.0 + 1, N::default());
        }
        self.nodes[v.0] += val;
    }

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