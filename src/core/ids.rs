#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct NodeId(pub usize);

impl From<NodeId> for usize {
    #[inline]
    fn from(n: NodeId) -> Self {
        n.0
    }
}

impl NodeId {
    #[inline]
    pub fn index(self) -> usize {
        self.0
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct EdgeId(pub usize);

impl EdgeId {
    #[inline]
    pub fn index(self) -> usize {
        self.0
    }
}
