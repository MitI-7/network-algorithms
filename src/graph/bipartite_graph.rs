use crate::graph::{
    direction::{Directed, Direction, Undirected},
    edge::BipartiteEdge,
    ids::{EdgeId, LeftNodeId, RightNodeId},
    node::Node,
};
use std::marker::PhantomData;

#[derive(Clone, Debug)]
pub struct BipartiteGraph<D, N = (), E = ()> {
    left_nodes: Vec<Node<N>>,
    right_nodes: Vec<Node<N>>,
    pub(crate) edges: Vec<BipartiteEdge<E>>,
    pub(crate) degree_left: Vec<usize>,
    pub(crate) degree_right: Vec<usize>,
    _direction: PhantomData<D>,
}

impl<D: Direction, N, E> BipartiteGraph<D, N, E> {
    #[inline]
    pub fn num_left_nodes(&self) -> usize {
        self.left_nodes.len()
    }

    pub fn num_right_nodes(&self) -> usize {
        self.right_nodes.len()
    }

    #[inline]
    pub fn num_edges(&self) -> usize {
        self.edges.len()
    }

    pub fn add_left_node_with(&mut self, data: N) -> LeftNodeId {
        let node_id = LeftNodeId(self.num_left_nodes());
        self.left_nodes.push(Node { data });
        self.degree_left.push(0);
        node_id
    }

    pub fn add_left_nodes_with<I>(&mut self, datas: I) -> Vec<LeftNodeId>
    where
        I: IntoIterator<Item = N>,
    {
        datas.into_iter().map(|d| self.add_left_node_with(d)).collect()
    }

    pub fn add_right_node_with(&mut self, data: N) -> RightNodeId {
        let node_id = RightNodeId(self.num_right_nodes());
        self.right_nodes.push(Node { data });
        self.degree_right.push(0);
        node_id
    }

    pub fn add_right_nodes_with<I>(&mut self, datas: I) -> Vec<RightNodeId>
    where
        I: IntoIterator<Item = N>,
    {
        datas.into_iter().map(|d| self.add_right_node_with(d)).collect()
    }

    // TODO: r -> lの有向辺をどうするか
    // いまは無向辺しか表現できない
    // pub fn add_directed_edge(&mut self, from: NodeId, to: NodeId, data: E) -> EdgeId {
    //     self.edges.push(Edge { u: from, v: to, data });
    //     self.degree_left[from.index()] += 1;
    //     self.degree_right[to.index()] += 1;
    //     EdgeId(self.edges.len() - 1)
    // }

    pub fn add_edge(&mut self, u: LeftNodeId, v: RightNodeId, data: E) -> Option<EdgeId> {
        if u.index() >= self.num_left_nodes() || v.index() >= self.num_right_nodes() {
            return None;
        }
        let edge_id = EdgeId(self.edges.len());
        self.edges.push(BipartiteEdge { u, v, data });
        self.degree_left[u.index()] += 1;
        self.degree_right[v.index()] += 1;
        Some(edge_id)
    }

    pub fn get_left_node(&self, node_id: LeftNodeId) -> Option<&Node<N>> {
        if node_id.index() >= self.num_left_nodes() {
            return None;
        }
        Some(&self.left_nodes[node_id.index()])
    }

    pub fn get_left_node_mut(&mut self, node_id: LeftNodeId) -> Option<&mut Node<N>> {
        if node_id.index() >= self.num_left_nodes() {
            return None;
        }
        Some(&mut self.left_nodes[node_id.index()])
    }

    pub fn get_right_node(&self, node_id: RightNodeId) -> Option<&Node<N>> {
        if node_id.index() >= self.num_right_nodes() {
            return None;
        }
        Some(&self.right_nodes[node_id.index()])
    }

    pub fn get_right_node_mut(&mut self, node_id: RightNodeId) -> Option<&mut Node<N>> {
        if node_id.index() >= self.num_right_nodes() {
            return None;
        }
        Some(&mut self.right_nodes[node_id.index()])
    }

    pub fn get_edge(&self, edge_id: EdgeId) -> Option<&BipartiteEdge<E>> {
        if edge_id.index() >= self.edges.len() {
            return None;
        }
        Some(&self.edges[edge_id.index()])
    }

    pub fn left_nodes(&self) -> std::slice::Iter<'_, Node<N>> {
        self.left_nodes.iter()
    }

    pub fn right_nodes(&self) -> std::slice::Iter<'_, Node<N>> {
        self.right_nodes.iter()
    }

    pub fn edges(&self) -> std::slice::Iter<'_, BipartiteEdge<E>> {
        self.edges.iter()
    }
}

impl<D: Direction, N: Default, E> BipartiteGraph<D, N, E> {
    pub fn add_left_node(&mut self) -> LeftNodeId {
        self.add_left_node_with(N::default())
    }

    pub fn add_left_nodes(&mut self, n: usize) -> Vec<LeftNodeId> {
        (0..n).map(|_| self.add_left_node()).collect()
    }

    pub fn add_right_node(&mut self) -> RightNodeId {
        self.add_right_node_with(N::default())
    }

    pub fn add_right_nodes(&mut self, n: usize) -> Vec<RightNodeId> {
        (0..n).map(|_| self.add_right_node()).collect()
    }
}

impl<D: Direction, N, E> Default for BipartiteGraph<D, N, E> {
    fn default() -> Self {
        Self {
            left_nodes: Vec::new(),
            right_nodes: Vec::new(),
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
