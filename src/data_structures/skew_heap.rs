//! Skew Heap (priority‑queue style) — supports `(key, value)` pairs
//! ================================================================
//! * **Max‑heap** by default (largest key has highest priority).
//! * To obtain a **min‑heap** and keep lazy `add_all()` working, wrap the key in
//!   the provided `Rev<K>` new‑type.
//! * Constant‑time `add_all(Δ)` adds `Δ` to **all keys**, not the values.
//! * API intentionally resembles the community crate **`priority‑queue`**, with
//!   one deviation: **`peek()` returns a cloned `(K, V)`** instead of references
//!   to avoid lifetime issues when lazy propagation mutates internal state.
//!
//! ### Trait bounds
//! | Type | Bounds |
//! |------|--------|
//! | `K` (key)   | `Ord + Copy + Add<Output=K> + AddAssign + Default` |
//! | `V` (value) | `Clone` *(needed by `peek()` to return an owned value)* |
//!
//! Works out‑of‑the‑box for numeric keys (`i*`, `u*`, `f*`, `Duration`, …).  For
//! min‑heap semantics use `Rev<K>`.
//!
//! ---
//! ## Usage example
//! ```rust
//! use skew_heap::{SkewHeap, Rev};
//!
//! // ---------- max‑heap ----------
//! let mut pq = SkewHeap::<i32, &str>::new();
//! pq.push(3, "low");
//! pq.push(7, "high");
//! pq.push(5, "medium");
//! assert_eq!(pq.peek(), Some((7, "high"))); // cloned pair
//! pq.add_all(10);          // priorities +10
//! assert_eq!(pq.pop(), Some((17, "high")));
//!
//! // ---------- min‑heap with Rev ----------
//! let mut min_pq = SkewHeap::<Rev<i32>, &str>::new();
//! min_pq.push(Rev(3), "low");
//! min_pq.push(Rev(7), "high");
//! assert_eq!(min_pq.pop().map(|(k, v)| (k.0, v)), Some((3, "low")));
//! ```
//!
//! ---
//! ### Complexity
//! | operation | time | notes |
//! |-----------|------|-------|
//! | `push` / `pop` | *O(log n)* amortised | |
//! | `peek` | *O(1)* | returns cloned `(K, V)` |
//! | `merge_with` | *O(log n)* amortised | merges two queues |
//! | `add_all(Δ)` | **O(1)** | lazy propagation to keys |
//!
//! ---
//! Author: ChatGPT (OpenAI) – 2025‑06‑24

use core::cmp::Ordering;
use core::ops::{Add, AddAssign};

//──────────────────────────────────────────────────────────────────────────────
// Helper new‑type: Rev<K>  (reverse order + arithmetic passthrough)
//──────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Rev<K>(pub K);

impl<K: Ord> Ord for Rev<K> {
    #[inline] fn cmp(&self, other: &Self) -> Ordering { other.0.cmp(&self.0) }
}
impl<K: PartialOrd> PartialOrd for Rev<K> {
    #[inline] fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.0.partial_cmp(&self.0)
    }
}
// Forward arithmetic so that `add_all()` works on Rev keys.
impl<K> Add for Rev<K>
where
    K: Add<Output = K>,
{
    type Output = Self;
    #[inline] fn add(self, rhs: Self) -> Self { Rev(self.0 + rhs.0) }
}
impl<K> AddAssign for Rev<K>
where
    K: AddAssign,
{
    #[inline] fn add_assign(&mut self, rhs: Self) { self.0 += rhs.0; }
}
impl<K> Add<K> for Rev<K>
where
    K: Add<Output = K> + Copy,
{
    type Output = Self;
    #[inline] fn add(self, rhs: K) -> Self { Rev(self.0 + rhs) }
}
impl<K> AddAssign<K> for Rev<K>
where
    K: AddAssign + Copy,
{
    #[inline] fn add_assign(&mut self, rhs: K) { self.0 += rhs; }
}

//──────────────────────────────────────────────────────────────────────────────
// Core data structures
//──────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct SkewHeap<K, V>
where
    K: Ord + Copy + Add<Output = K> + AddAssign + Default,
    V: Clone,
{
    root: Option<Box<Node<K, V>>>,
}

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
    lazy: K, // pending Δ to add to *all keys* in this subtree
}

impl<K, V> Node<K, V>
where
    K: Ord + Copy + Add<Output = K> + AddAssign + Default,
    V: Clone,
{
    #[inline] fn new(key: K, val: V) -> Self {
        Self { key, val, left: None, right: None, lazy: K::default() }
    }

    /// Push down any pending Δ, making `key` current.
    #[inline]
    fn propagate(&mut self) {
        if self.lazy != K::default() {
            self.key += self.lazy;
            if let Some(ref mut l) = self.left  { l.lazy += self.lazy; }
            if let Some(ref mut r) = self.right { r.lazy += self.lazy; }
            self.lazy = K::default();
        }
    }
}

//──────────────────────────────────────────────────────────────────────────────
// Public API — mirrors crate `priority_queue` (mod `peek` diff)
//──────────────────────────────────────────────────────────────────────────────

impl<K, V> SkewHeap<K, V>
where
    K: Ord + Copy + Add<Output = K> + AddAssign + Default,
    V: Clone,
{
    /// Create an empty queue.
    #[inline] pub fn new() -> Self { Self { root: None } }

    /// Is the queue empty?
    #[inline] pub fn is_empty(&self) -> bool { self.root.is_none() }

    /// Insert (`key`, `value`).
    pub fn push(&mut self, key: K, val: V) {
        let node = Some(Box::new(Node::new(key, val)));
        self.root = Self::merge_nodes(self.root.take(), node);
    }

    /// Peek highest‑priority entry without removal.
    /// Returns a **cloned** `(key, value)` pair.
    pub fn peek(&mut self) -> Option<(K, V)> {
        if let Some(ref mut r) = self.root {
            r.propagate();
            Some((r.key, r.val.clone()))
        } else {
            None
        }
    }

    /// Pop and return `(key, value)` of the highest priority.
    pub fn pop(&mut self) -> Option<(K, V)> {
        let mut root = self.root.take()?;
        root.propagate();
        let left  = root.left.take();
        let right = root.right.take();
        let result = (root.key, root.val);
        self.root = Self::merge_nodes(left, right);
        Some(result)
    }

    /// In‑place **destructive** merge — empties `other`.
    pub fn merge_with(&mut self, mut other: Self) {
        self.root = Self::merge_nodes(self.root.take(), other.root.take());
    }

    /// Lazily add `Δ` to *all keys* currently stored (O(1)).
    pub fn add_all(&mut self, delta: K) {
        if let Some(ref mut r) = self.root {
            r.lazy += delta;
        }
    }

    //──────────────────────── internal helpers ──────────────────────────────

    fn merge_nodes(
        mut h1: Option<Box<Node<K, V>>>,
        mut h2: Option<Box<Node<K, V>>>,
    ) -> Option<Box<Node<K, V>>> {
        match (h1.as_mut(), h2.as_mut()) {
            (None, None)       => return None,
            (Some(_), None)    => return h1,
            (None, Some(_))    => return h2,
            (Some(n1), Some(n2)) => {
                n1.propagate();
                n2.propagate();
                // Keep `n1` as the larger root (max‑heap invariant)
                if n2.key > n1.key {
                    core::mem::swap(&mut h1, &mut h2);
                }
            }
        }

        // Skew step: swap children then merge.
        if let Some(mut root) = h1 {
            let right = root.right.take();
            root.right = root.left.take();
            root.left  = Self::merge_nodes(right, h2);
            Some(root)
        } else {
            h2
        }
    }
}

//──────────────────────────────────────────────────────────────────────────────
// Unit tests
//──────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::{Rev, SkewHeap};

    #[test]
    fn basic_max_heap() {
        let mut pq = SkewHeap::<i32, &str>::new();
        pq.push(3, "low");
        pq.push(7, "high");
        pq.push(5, "medium");
        assert_eq!(pq.peek(), Some((7, "high")));
        pq.add_all(10);
        assert_eq!(pq.pop(), Some((17, "high")));
        assert_eq!(pq.pop(), Some((15, "medium")));
        assert_eq!(pq.pop(), Some((13, "low")));
        assert!(pq.is_empty());
    }

    #[test]
    fn basic_min_heap_with_rev() {
        let mut pq = SkewHeap::<Rev<i32>, &str>::new();
        pq.push(Rev(7), "high");
        pq.push(Rev(3), "low");
        pq.push(Rev(5), "medium");
        assert_eq!(pq.peek().map(|(k, v)| (k.0, v)), Some((3, "low")));
        pq.add_all(Rev(10));
        assert_eq!(pq.pop().map(|(k, v)| (k.0, v)), Some((13, "low")));
        assert_eq!(pq.pop().map(|(k, v)| (k.0, v)), Some((15, "medium")));
        assert_eq!(pq.pop().map(|(k, v)| (k.0, v)), Some((17, "high")));
    }

    #[test]
    fn merge_queues() {
        let mut a = SkewHeap::<i32, &str>::new();
        a.push(4, "a4");
        a.push(1, "a1");
        let mut b = SkewHeap::<i32, &str>::new();
        b.push(3, "b3");
        b.push(2, "b2");
        a.merge_with(b);
        // assert_eq!(a.pop(), Some((4

    }
}
        