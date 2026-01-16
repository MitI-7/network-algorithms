use crate::graph::ids::{LeftNodeId, NodeId, RightNodeId};

#[derive(Clone, Copy, Debug)]
pub struct Edge<E> {
    pub u: NodeId,
    pub v: NodeId,
    pub data: E,
}

#[derive(Clone, Copy, Debug)]
pub struct BipartiteEdge<E> {
    pub u: LeftNodeId,
    pub v: RightNodeId,
    pub data: E,
}
