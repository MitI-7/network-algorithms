use crate::graph::ids::ArcId;

pub(crate) struct ArcIdRange {
    pub(crate) cur: usize,
    pub(crate) end: usize,
}

impl Iterator for ArcIdRange {
    type Item = ArcId;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.cur >= self.end {
            return None;
        }
        let a = ArcId(self.cur);
        self.cur += 1;
        Some(a)
    }
}
