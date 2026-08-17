/* Copyright 2024-2026 Joachim Metz <joachim.metz@gmail.com>
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
use std::sync::RwLock;

use keramics_core::ErrorTrace;

/// LRU cache entry.
struct LruCacheEntry<V> {
    /// Value.
    pub value: V,
}

impl<V> LruCacheEntry<V> {
    /// Creates a new cache entry.
    pub fn new(value: V) -> Self {
        Self { value }
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

impl<K: Hash + Eq + Clone, V> LruCache<K, V> {
    /// Creates a new cache.
    pub fn new(number_of_entries: usize) -> Self {
        Self {
            number_of_entries,
            values: HashMap::new(),
            usage: VecDeque::new(),
        }
    }

    /// Determines if a specific value is cached.
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

    /// Retrieves a specific mutable value from the cache.
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        match self.values.get_mut(key) {
            Some(entry) => Some(&mut (*entry).value),
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
        self.values.insert(key.clone(), entry);
        self.usage.push_back(key);
    }
}

/// Variant of LRU cache that can be shared across threads.
pub struct SharedLruCache<K: Hash + Eq, V> {
    /// Cache.
    cache: RwLock<LruCache<K, V>>,
}

impl<K: Hash + Eq + Clone, V: Clone> SharedLruCache<K, V> {
    /// Creates a new cache.
    pub fn new(number_of_entries: usize) -> Self {
        Self {
            cache: RwLock::new(LruCache::new(number_of_entries)),
        }
    }

    /// Determines if a specific value is cached.
    pub fn contains(&self, key: &K) -> Result<bool, ErrorTrace> {
        match self.cache.read() {
            Ok(cache) => Ok(cache.contains(key)),
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to obtain read lock on cache",
                    error
                ));
            }
        }
    }

    /// Retrieves a specific value from the cache.
    pub fn get(&self, key: &K) -> Result<Option<V>, ErrorTrace> {
        match self.cache.write() {
            Ok(mut cache) => Ok(cache.get(key).cloned()),
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to obtain write lock on cache",
                    error
                ));
            }
        }
    }

    /// Inserts a specific value into the cache.
    pub fn insert(&self, key: K, value: V) -> Result<(), ErrorTrace> {
        match self.cache.write() {
            Ok(mut cache) => Ok(cache.insert(key, value)),
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to obtain write lock on cache",
                    error
                ));
            }
        }
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
        lru_cache.insert(1, String::from("test1"));

        let result: Option<&String> = lru_cache.get(&1);
        assert!(result.is_some());
        assert_eq!(result.unwrap().as_str(), "test1");

        let result: Option<&String> = lru_cache.get(&99);
        assert!(result.is_none());
    }

    #[test]
    fn test_get_mut() {
        let mut lru_cache: LruCache<usize, String> = LruCache::new(2);
        lru_cache.insert(1, String::from("test1"));

        let result: Option<&mut String> = lru_cache.get_mut(&1);
        assert!(result.is_some());
        assert_eq!(result.unwrap().as_str(), "test1");

        let result: Option<&mut String> = lru_cache.get_mut(&99);
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
