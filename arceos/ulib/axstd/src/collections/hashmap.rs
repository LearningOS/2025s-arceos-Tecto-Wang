use alloc::vec::Vec;
use core::hash::{BuildHasher, BuildHasherDefault, Hash, Hasher};

#[derive(Default)]
pub struct SimpleHasher(u64);
impl Hasher for SimpleHasher {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 = self.0.wrapping_mul(31).wrapping_add(*byte as u64);
        }
    }
}

type DefaultHasher = BuildHasherDefault<SimpleHasher>;

#[derive(Debug)]
pub struct HashMap<K, V> {
    buckets: Vec<Option<(K, V)>>,
    hasher: DefaultHasher,
    size: usize,
}

impl<K: Eq + Hash, V> HashMap<K, V> {
    pub fn new() -> Self {
        let mut buckets = Vec::with_capacity(64);
        buckets.resize_with(64, || None);
        Self {
            buckets,
            hasher: DefaultHasher::default(),
            size: 0,
        }
    }

    fn hash(&self, key: &K) -> usize {
        let mut hasher = self.hasher.build_hasher();
        key.hash(&mut hasher);
        (hasher.finish() % self.buckets.len() as u64) as usize
    }

    pub fn insert(&mut self, key: K, value: V) {
        // Check load factor: if size / capacity > 0.7, resize
        if self.size * 10 >= self.buckets.len() * 7 {
            self.resize();
        }

        let mut index = self.hash(&key);
        for _ in 0..self.buckets.len() {
            match &self.buckets[index] {
                None => {
                    self.buckets[index] = Some((key, value));
                    self.size += 1;
                    return;
                }
                Some((k, _)) if *k == key => {
                    self.buckets[index] = Some((key, value));
                    return;
                }
                _ => {
                    index = (index + 1) % self.buckets.len();
                }
            }
        }
        panic!("HashMap full!");
    }

    fn resize(&mut self) {
        let new_capacity = self.buckets.len() * 2;
        let mut new_buckets = Vec::with_capacity(new_capacity);
        new_buckets.resize_with(new_capacity, || None);

        let old_buckets = core::mem::replace(&mut self.buckets, new_buckets);
        self.size = 0;

        for entry in old_buckets.into_iter().flatten() {
            self.insert(entry.0, entry.1);
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let mut index = self.hash(key);
        for _ in 0..self.buckets.len() {
            match &self.buckets[index] {
                Some((k, v)) if k == key => return Some(v),
                None => return None,
                _ => {
                    index = (index + 1) % self.buckets.len();
                }
            }
        }
        None
    }

    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter {
            inner: self.buckets.iter(),
        }
    }
}

pub struct Iter<'a, K, V> {
    inner: core::slice::Iter<'a, Option<(K, V)>>,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(item) = self.inner.next() {
            if let Some((ref k, ref v)) = item {
                return Some((k, v));
            }
        }
        None
    }
}