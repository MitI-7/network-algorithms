#[derive(Clone)]
pub struct SkewHeap<W> {
    left: Option<Box<SkewHeap<W>>>,
    right: Option<Box<SkewHeap<W>>>,
    id: usize,
    lazy: W,
    val: W,
}

impl<W> SkewHeap<W>
where
    W: Copy + Default + std::ops::Add<Output = W> + std::ops::AddAssign + PartialOrd,
{
    pub fn new(v: W, id: usize) -> Box<SkewHeap<W>> {
        Box::new(SkewHeap { left: None, right: None, id, lazy: W::default(), val: v })
    }

    pub fn pop(mut self: Box<SkewHeap<W>>) -> Option<Box<SkewHeap<W>>> {
        self.push_lazy();
        SkewHeap::meld(self.left.take(), self.right.take())
    }

    pub fn peek_id(&mut self) -> usize {
        self.id
    }

    pub fn peek_key(&mut self) -> W {
        self.val + self.lazy
    }

    pub fn add_all(&mut self, delta: W) {
        self.lazy += delta;
    }

    pub fn meld(a: Option<Box<SkewHeap<W>>>, b: Option<Box<SkewHeap<W>>>) -> Option<Box<SkewHeap<W>>> {
        match (a, b) {
            (None, h) | (h, None) => h,
            (Some(mut ha), Some(mut hb)) => {
                ha.push_lazy();
                hb.push_lazy();
                if ha.val > hb.val {
                    std::mem::swap(&mut ha, &mut hb);
                }
                let merged = SkewHeap::meld(ha.right.take(), Some(hb));
                ha.right = merged;
                std::mem::swap(&mut ha.left, &mut ha.right);
                Some(ha)
            }
        }
    }

    fn push_lazy(&mut self) {
        if self.lazy != W::default() {
            if let Some(ref mut l) = self.left {
                l.lazy += self.lazy;
            }
            if let Some(ref mut r) = self.right {
                r.lazy += self.lazy;
            }
            self.val += self.lazy;
            self.lazy = W::default();
        }
    }
}
