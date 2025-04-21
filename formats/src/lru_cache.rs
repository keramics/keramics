/* Copyright 2024-2025 Joachim Metz <joachim.metz@gmail.com>
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License. You may
 * obtain a copy of the License at https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
 * WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
 * License for the specific language governing permissions and limitations
 * under the License.
 */

use std::collections::{HashMap, VecDeque};
use std::hash::Hash;

/// LRU cache entry.
struct LruCacheEntry<V> {
    /// Value.
    pub value: V,
}

impl<V> LruCacheEntry<V> {
    /// Creates a new cache entry.
    pub fn new(value: V) -> Self {
        Self { value: value }
    }
}

/// LRU cache.
pub struct LruCache<K: Hash + Eq, V> {
    /// Number of entries.
    number_of_entries: usize,

    /// Values.
    values: HashMap<K, LruCacheEntry<V>>,

    /// Usage.
    usage: VecDeque<K>,
}

impl<K: Hash + Eq + Copy, V> LruCache<K, V> {
    /// Creates a new cache.
    pub fn new(number_of_entries: usize) -> Self {
        Self {
            number_of_entries: number_of_entries,
            values: HashMap::new(),
            usage: VecDeque::new(),
        }
    }

    /// Determines if a specific valud is caches.
    pub fn contains(&self, key: &K) -> bool {
        self.values.contains_key(key)
    }

    /// Retrieves a specific value from the cache.
    pub fn get(&mut self, key: &K) -> Option<&V> {
        match self.values.get(key) {
            Some(entry) => Some(&(*entry).value),
            None => None,
        }
    }

    /// Inserts a specific value into the cache.
    pub fn insert(&mut self, key: K, value: V) {
        if self.usage.len() == self.number_of_entries {
            let lru_key: K = self.usage.pop_front().unwrap();
            self.values.remove(&lru_key);
        }
        let entry: LruCacheEntry<V> = LruCacheEntry::new(value);
        self.values.insert(key, entry);
        self.usage.push_back(key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains() {
        let mut lru_cache: LruCache<usize, &str> = LruCache::new(2);
        lru_cache.insert(1, "test1");

        assert_eq!(lru_cache.contains(&1), true);

        assert_eq!(lru_cache.contains(&99), false);
    }

    #[test]
    fn test_get() {
        let mut lru_cache: LruCache<usize, String> = LruCache::new(2);
        lru_cache.insert(1, "test1".to_string());

        let result: Option<&String> = lru_cache.get(&1);
        assert!(result.is_some());
        assert_eq!(result.unwrap().as_str(), "test1");

        let result: Option<&String> = lru_cache.get(&99);
        assert!(result.is_none());
    }

    #[test]
    fn test_insert() {
        let mut lru_cache: LruCache<usize, &str> = LruCache::new(2);

        assert_eq!(lru_cache.values.len(), 0);
        assert_eq!(lru_cache.usage.len(), 0);

        lru_cache.insert(1, "test1");

        assert_eq!(lru_cache.values.len(), 1);
        assert_eq!(lru_cache.usage.len(), 1);
        assert_eq!(lru_cache.usage, [1]);

        lru_cache.insert(2, "test2");

        assert_eq!(lru_cache.values.len(), 2);
        assert_eq!(lru_cache.usage.len(), 2);
        assert_eq!(lru_cache.usage, [1, 2]);

        lru_cache.insert(3, "test3");

        assert_eq!(lru_cache.values.len(), 2);
        assert_eq!(lru_cache.usage.len(), 2);
        assert_eq!(lru_cache.usage, [2, 3]);
    }
}
