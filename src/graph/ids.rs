#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct NodeId(pub usize);

// impl From<NodeId> for usize {
//     #[inline(always)]
//     fn from(n: NodeId) -> Self {
//         n.0
//     }
// }

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
