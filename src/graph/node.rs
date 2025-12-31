use crate::graph::ids::NodeId;

#[derive(Clone, Debug)]
pub struct Node<N> {
    pub u: NodeId,
    pub data: N,
}
