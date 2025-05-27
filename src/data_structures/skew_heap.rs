#[derive(Clone)]
pub struct SkewHeap<K> {
    id: usize, // Index in the 'edges' Vec from 'msa'
    lazy: K,
    key: K,
    left: Option<Box<SkewHeap<K>>>,
    right: Option<Box<SkewHeap<K>>>,
}

impl<K> SkewHeap<K>
where
    K: Copy + Default + std::ops::Add<Output = K> + std::ops::AddAssign + PartialOrd,
{
    pub fn new(v: K, id: usize) -> Box<SkewHeap<K>> {
        Box::new(SkewHeap { id, lazy: K::default(), key: v, left: None, right: None })
    }

    pub fn pop(mut self: Box<SkewHeap<K>>) -> Option<Box<SkewHeap<K>>> {
        self.push_lazy();
        SkewHeap::meld(self.left.take(), self.right.take())
    }

    pub fn peek_id(&mut self) -> usize {
        // Renamed from peek_id to avoid confusion if an EdgeId field were added
        self.id
    }

    pub fn peek_key(&mut self) -> K {
        self.key + self.lazy
    }

    pub fn add_all(&mut self, delta: K) {
        self.lazy += delta;
    }

    pub fn meld(a: Option<Box<SkewHeap<K>>>, b: Option<Box<SkewHeap<K>>>) -> Option<Box<SkewHeap<K>>> {
        match (a, b) {
            (None, h) | (h, None) => h,
            (Some(mut ha), Some(mut hb)) => {
                ha.push_lazy();
                hb.push_lazy();
                if ha.key > hb.key {
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
        if self.lazy != K::default() {
            if let Some(ref mut l) = self.left {
                l.lazy += self.lazy;
            }
            if let Some(ref mut r) = self.right {
                r.lazy += self.lazy;
            }
            self.key += self.lazy;
            self.lazy = K::default();
        }
    }
}
