#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct NodeId(pub usize);

impl NodeId {
    #[inline(always)]
    pub fn index(self) -> usize {
        self.0
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct EdgeId(pub usize);

impl EdgeId {
    #[inline(always)]
    pub fn index(self) -> usize {
        self.0
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ArcId(pub usize);

impl ArcId {
    #[inline(always)]
    pub fn index(self) -> usize {
        self.0
    }
}

pub const INVALID_NODE_ID: NodeId = NodeId(usize::MAX);
pub const INVALID_EDGE_ID: EdgeId = EdgeId(usize::MAX);
pub const INVALID_ARC_ID: ArcId = ArcId(usize::MAX);