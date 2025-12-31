use crate::graph::ids::NodeId;

#[derive(Clone, Copy, Debug)]
pub struct Edge<E> {
    pub u: NodeId,
    pub v: NodeId,
    pub data: E,
}
