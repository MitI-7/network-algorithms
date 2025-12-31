#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
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

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct EdgeId(pub usize);

impl EdgeId {
    #[inline]
    pub fn index(self) -> usize {
        self.0
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct ArcId(pub usize);

impl ArcId {
    #[inline]
    pub fn index(self) -> usize {
        self.0
    }
}
