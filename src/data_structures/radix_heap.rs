use std::mem;

pub struct RadixHeap<K, V> {
    buckets: Box<[Bucket<K, V>]>,
    len: usize,
}

struct Bucket<K, V> {
    start: K,
    data: Vec<(K, V)>,
}

#[allow(dead_code)]
impl<K, V> RadixHeap<K, V>
where
    K: RadixKey,
{
    // c: 同時にヒープに入るキーの差の最大値
    pub fn new(c: K) -> Self {
        assert!(c >= K::ZERO);
        let bit_width = size_of::<K>() * 8;
        let num_buckets = bit_width - c.leading_zeros() as usize + 3;
        Self {
            buckets: (0..num_buckets)
                .map(|i| Bucket { start: Self::size_sum(i), data: Vec::new() })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            len: 0,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    // O(log C)
    pub fn push(&mut self, key: K, value: V) {
        let bucket = self.buckets.iter_mut().rev().find(|b| b.start <= key).expect("monotonicity was violated.");
        bucket.data.push((key, value));
        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<(K, V)> {
        if self.len == 0 {
            return None;
        }

        if self.buckets[0].data.is_empty() {
            self.distribute();
        }

        self.len -= 1;
        self.buckets[0].data.pop()
    }

    fn distribute(&mut self) {
        assert!(self.len > 0);
        let first_non_empty_bucket_idx = self.buckets.iter().position(|b| !b.data.is_empty()).unwrap();

        let data = mem::take(&mut self.buckets[first_non_empty_bucket_idx].data);
        let min_key = data.iter().map(|&(key, _)| key).min().unwrap();

        // set bucket start
        let end = self.buckets.get(first_non_empty_bucket_idx + 1).map_or(K::MAX, |b| b.start);
        self.buckets[..=first_non_empty_bucket_idx].iter_mut().enumerate().for_each(|(i, b)| {
            b.start = end.min(min_key + Self::size_sum(i));
        });

        // distribute
        for item in data {
            self.buckets[..first_non_empty_bucket_idx]
                .iter_mut()
                .rev()
                .find(|b| b.start <= item.0)
                .unwrap()
                .data
                .push(item);
        }
    }

    fn size_sum(i: usize) -> K {
        match i {
            0 => K::ZERO,
            _ => K::ONE << (i - 1),
        }
    }
}

pub trait RadixKey: Copy + Ord + core::ops::Add<Output = Self> + core::ops::Shl<usize, Output = Self> {
    const ZERO: Self;
    const ONE: Self;
    const MAX: Self;
    const BITS: usize;

    fn leading_zeros(self) -> u32;
}

macro_rules! impl_radix_key {
    ($($t:ty),*) => {$(
        impl RadixKey for $t {
            const ZERO: Self = 0;
            const ONE:  Self = 1;
            const MAX:  Self = <$t>::MAX;
            const BITS: usize = mem::size_of::<Self>() * 8;

            #[inline]
            fn leading_zeros(self) -> u32 { <$t>::leading_zeros(self) }
        }
    )*};
}

impl_radix_key!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let mut heap = RadixHeap::<u64, i32>::new(5);

        heap.push(1, 0);
        heap.push(3, 1);
        heap.push(2, 2);
        assert_eq!(Some((1, 0)), heap.pop());

        heap.push(4, 3);
        heap.push(4, 3);
        assert_eq!(Some((2, 2)), heap.pop());
        assert_eq!(Some((3, 1)), heap.pop());
        assert_eq!(Some((4, 3)), heap.pop());
        assert_eq!(Some((4, 3)), heap.pop());
        assert_eq!(None, heap.pop());
    }
}
