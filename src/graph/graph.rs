use std::marker::PhantomData;
use std::ops::AddAssign;
use std::ops::{Index, IndexMut};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct NodeId(pub usize);

impl From<NodeId> for usize { #[inline] fn from(n: NodeId) -> Self { n.0 } }

impl NodeId {
    #[inline] pub fn index(self) -> usize { self.0 }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct EdgeId(pub usize);


pub trait Direction { const IS_DIRECTED: bool; }

#[derive(Clone, Copy)]
pub struct Directed;

#[derive(Clone, Copy)]
pub struct Undirected;

impl Direction for Directed   { const IS_DIRECTED: bool = true;  }

impl Direction for Undirected { const IS_DIRECTED: bool = false; }

/*====================================================
  Edge record (from, to, payload)
====================================================*/
#[derive(Clone, Debug)]
pub struct Edge<E> { pub from: NodeId, pub to: NodeId, pub data: E }

/*====================================================
  Core graph: only edge list + optional node data
====================================================*/
#[derive(Clone, Debug)]
pub struct Graph<D: Direction, N: Default + Clone = (), E: Clone = ()> {
    pub nodes: Vec<N>,     // one payload per vertex (excess etc.)
    pub edges: Vec<Edge<E>>,   // |E| or 2|E| when undirected
    _dir: PhantomData<D>,
}

impl<D: Direction, N: Default + Clone, E: Clone> Graph<D, N, E> {
    pub fn new() -> Self {
        Self { edges: Vec::new(), nodes: Vec::new(), _dir: PhantomData }
    }

    /*----------- vertex ops -----------*/
    pub fn add_node(&mut self) -> NodeId {
        let id = NodeId(self.nodes.len());
        self.nodes.push(N::default());
        id
    }
    pub fn add_nodes(&mut self, n: usize) -> Vec<NodeId> {
        (0..n).map(|_| self.add_node()).collect()
    }

    /*----------- edge ops -----------*/
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

    /*----------- optional node payload (excess etc.) -----------*/
    pub fn add_node_value(&mut self, v: NodeId, val: N)
    where
        N: AddAssign,
    {
        if v.0 >= self.nodes.len() {
            self.nodes.resize(v.0 + 1, N::default());
        }
        self.nodes[v.0] += val;
    }

    /*----------- trivial accessors -----------*/
    pub fn edges(&self) -> &[Edge<E>] { &self.edges }
    pub fn node_payload(&self) -> &[N] { &self.nodes }
    pub fn num_nodes(&self) -> usize { self.nodes.len() }
    pub fn num_edges(&self) -> usize { self.edges.len() }
}

/*====================================================
  Example edge payloads (can be any Copy/Clone struct)
====================================================*/
/// ―― 最短経路 ――
#[derive(Clone, Copy, Debug)]
pub struct WeightEdge<W> {
    pub weight: W
}

/// ―― 最大流／一般化最大流 ――
#[derive(Clone, Copy, Debug)]
pub struct CapEdge<F> {
    pub flow: F,
    pub upper: F,
}

/// ―― 最小費用流／一般化最小費用流 ――
#[derive(Clone, Copy, Debug)]
pub struct CapCostEdge<F> {
    pub flow: F,
    pub lower: F,
    pub upper: F,
    pub cost:  F,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ExcessNode<F> {
    pub b: F,
    pub excess: F,
}


// /*====================================================
//   Tests
// ====================================================*/
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn build_graph_edge_list() {
//         type N = usize;
//         type C = i64;
//         let mut g: Graph<Directed, FlowEdge<N, C>, i64> = Graph::new();
//
//         let s = g.add_node();
//         let t = g.add_node();
//         g.add_nodes(3); // add 3 more nodes
//
//         g.add_edge(s, t, FlowEdge { lower: 1, upper: 10, cost: -5 });
//         g.add_node_value(s, 10);
//         g.add_node_value(t, -10);
//
//         assert_eq!(g.node_count(), 5);
//         assert_eq!(g.edge_count(), 1);
//         assert_eq!(g.node_payload()[s.0], 10);
//     }
// }
