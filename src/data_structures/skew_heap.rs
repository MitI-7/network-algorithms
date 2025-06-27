use core::ops::{Add, AddAssign};

#[derive(Debug, Clone)]
struct Node<K, V>
where
    K: Ord + Copy + Add<Output = K> + AddAssign + Default,
    V: Clone,
{
    key: K,
    val: V,
    left: Option<Box<Node<K, V>>>,
    right: Option<Box<Node<K, V>>>,
    lazy: K, // pending delta to add to all keys in this subtree
}

impl<K, V> Node<K, V>
where
    K: Ord + Copy + Add<Output = K> + AddAssign + Default,
    V: Clone,
{
    #[inline]
    fn new(key: K, val: V) -> Self {
        Self { key, val, left: None, right: None, lazy: K::default() }
    }

    #[inline]
    fn propagate(&mut self) {
        if self.lazy != K::default() {
            self.key += self.lazy;
            if let Some(ref mut l) = self.left {
                l.lazy += self.lazy;
            }
            if let Some(ref mut r) = self.right {
                r.lazy += self.lazy;
            }
            self.lazy = K::default();
        }
    }
}

#[derive(Clone, Default)]
pub struct SkewHeap<K, V>
where
    K: Ord + Copy + Add<Output = K> + AddAssign + Default,
    V: Clone,
{
    root: Option<Box<Node<K, V>>>,
}

impl<K, V> SkewHeap<K, V>
where
    K: Ord + Copy + Add<Output = K> + AddAssign + Default,
    V: Clone,
{
    #[inline]
    pub fn new() -> Self {
        Self { root: None }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    // O(log n) amortised
    pub fn push(&mut self, key: K, val: V) {
        let node = Some(Box::new(Node::new(key, val)));
        self.root = Self::merge_nodes(self.root.take(), node);
    }

    // O(1)
    pub fn peek(&mut self) -> Option<(K, V)> {
        if let Some(ref mut r) = self.root {
            r.propagate();
            Some((r.key, r.val.clone()))
        } else {
            None
        }
    }

    // O(log n) amortised
    pub fn pop(&mut self) -> Option<(K, V)> {
        let mut root = self.root.take()?;
        root.propagate();
        let left = root.left.take();
        let right = root.right.take();
        let result = (root.key, root.val);
        self.root = Self::merge_nodes(left, right);
        Some(result)
    }

    // O(log n) amortised
    pub fn merge_with(&mut self, mut other: Self) {
        self.root = Self::merge_nodes(self.root.take(), other.root.take());
    }

    // O(1)
    pub fn add_all(&mut self, delta: K) {
        if let Some(ref mut r) = self.root {
            r.lazy += delta;
        }
    }

    fn merge_nodes(mut h1: Option<Box<Node<K, V>>>, mut h2: Option<Box<Node<K, V>>>) -> Option<Box<Node<K, V>>> {
        match (h1.as_mut(), h2.as_mut()) {
            (None, None) => return None,
            (Some(_), None) => return h1,
            (None, Some(_)) => return h2,
            (Some(n1), Some(n2)) => {
                n1.propagate();
                n2.propagate();
                // Keep n1 as the larger root (max‑heap invariant)
                if n2.key > n1.key {
                    core::mem::swap(&mut h1, &mut h2);
                }
            }
        }

        // Skew step: swap children then merge.
        if let Some(mut root) = h1 {
            let right = root.right.take();
            root.right = root.left.take();
            root.left = Self::merge_nodes(right, h2);
            Some(root)
        } else {
            h2
        }
    }
}
