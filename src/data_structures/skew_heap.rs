use crate::traits::{IntNum, Zero};

// heap.rs
/// スキュー・ヒープに遅延加算機能を持たせた実装
/// `v` がキー、`id` が対応する辺の添え字
#[derive(Clone)]
pub struct SkewHeap<W> {
    left: Option<Box<SkewHeap<W>>>,
    right: Option<Box<SkewHeap<W>>>,
    pub add: W,
    pub v: W,
    pub id: usize,
}

impl<W> SkewHeap<W> 
where W: IntNum + Zero {
    /// 新しいノードを作る
    pub fn new(v: W, id: usize) -> Box<SkewHeap<W>> {
        Box::new(SkewHeap { left: None, right: None, add: W::zero(), v, id })
    }

    /// 自分に溜まった `add` を子に伝搬し、自身の `v` に反映
    pub fn push_lazy(&mut self) {
        if self.add != W::zero() {
            if let Some(ref mut l) = self.left {
                l.add += self.add;
            }
            if let Some(ref mut r) = self.right {
                r.add += self.add;
            }
            self.v += self.add;
            self.add = W::zero();
        }
    }

    /// 2 つのヒープをマージして返す
    pub fn meld(a: Option<Box<SkewHeap<W>>>, b: Option<Box<SkewHeap<W>>>) -> Option<Box<SkewHeap<W>>> {
        match (a, b) {
            (None, h) | (h, None) => h,
            (Some(mut ha), Some(mut hb)) => {
                // キーが小さい方を ha にする
                ha.push_lazy();
                hb.push_lazy();
                if ha.v > hb.v {
                    std::mem::swap(&mut ha, &mut hb);
                }
                // ha が親、右側を meld、左右交換
                let merged = SkewHeap::meld(ha.right.take(), Some(hb));
                ha.right = merged;
                std::mem::swap(&mut ha.left, &mut ha.right);
                Some(ha)
            }
        }
    }

    /// ルート要素（最小要素）を取り除いた残りのヒープを返す
    pub fn pop(mut self: Box<SkewHeap<W>>) -> Option<Box<SkewHeap<W>>> {
        self.push_lazy();
        SkewHeap::meld(self.left.take(), self.right.take())
    }

    /// 現在のルート要素のキー（コスト）を取得
    pub fn peek_key(&self) -> W {
        self.v + self.add
    }

    /// 現在のルート要素の id（辺添え字）を取得
    pub fn peek_id(&self) -> usize {
        self.id
    }

    /// ヒープ全体に一様に `delta` を加算（遅延）
    pub fn add_all(&mut self, delta: W) {
        self.add += delta;
    }
}
