#[derive(Clone)]
pub struct Node<K, V> {
    left: Option<usize>,
    right: Option<usize>,
    pub key: K,
    lazy: K,
    pub val: V,
}

pub struct SkewHeap<K, V> {
    nodes: Vec<Node<K, V>>,
}

impl<K, V> SkewHeap<K, V>
where
    K: Default + Copy + std::ops::Neg<Output = K>  + std::ops::SubAssign + std::ops::AddAssign + PartialEq + PartialOrd,
{
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self { nodes: Vec::with_capacity(capacity) }
    }

    pub fn add_node(&mut self, key: K, value: V) -> usize {
        self.nodes.push(Node { left: None, right: None, key, lazy: K::default(), val: value });
        self.nodes.len() - 1
    }

    pub fn push(&mut self, i: usize) {
        let lz = self.nodes[i].lazy;
        if lz != K::default() {
            let (l, r) = (self.nodes[i].left, self.nodes[i].right);
            self.nodes[i].lazy = K::default();
            if let Some(c) = l {
                self.add(c, lz);
            }
            if let Some(c) = r {
                self.add(c, lz);
            }
        }
    }

    pub fn add(&mut self, i: usize, d: K) {
        self.nodes[i].key -= d;
        self.nodes[i].lazy += d;
    }

    pub fn merge(&mut self, a: Option<usize>, b: Option<usize>) -> Option<usize> {
        match (a, b) {
            (None, None) => None,
            (Some(x), None) | (None, Some(x)) => Some(x),
            (Some(mut u), Some(mut v)) => {
                if self.nodes[v].key < self.nodes[u].key {
                    std::mem::swap(&mut u, &mut v);
                }
                self.push(u);
                let right = self.nodes[u].right;
                self.nodes[u].right = self.merge(right, Some(v));
                /* swap children */
                let tmp = self.nodes[u].left;
                self.nodes[u].left = self.nodes[u].right;
                self.nodes[u].right = tmp;
                Some(u)
            }
        }
    }

    pub fn pop(&mut self, root: &mut Option<usize>) {
        if let Some(r) = *root {
            self.push(r);
            *root = self.merge(self.nodes[r].left, self.nodes[r].right);
        }
    }

    pub fn get_node(&self, i: usize) -> &Node<K, V> {
        &self.nodes[i]
    }
}