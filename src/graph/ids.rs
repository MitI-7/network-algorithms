#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct NodeId(pub(crate) usize);

impl NodeId {
    #[inline(always)]
    pub fn index(self) -> usize {
        self.0
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LeftNodeId(pub(crate) usize);
impl LeftNodeId {
    #[inline(always)]
    pub fn index(self) -> usize {
        self.0
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RightNodeId(pub(crate) usize);

impl RightNodeId {
    #[inline(always)]
    pub fn index(self) -> usize {
        self.0
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct EdgeId(pub(crate) usize);

impl EdgeId {
    #[inline(always)]
    pub fn index(self) -> usize {
        self.0
    }
}

impl Default for EdgeId {
    fn default() -> Self {
        Self(usize::MAX)
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
pub(crate) const INVALID_ARC_ID: ArcId = ArcId(usize::MAX);
