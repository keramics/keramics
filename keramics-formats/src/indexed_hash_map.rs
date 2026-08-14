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

use std::collections::HashMap;
use std::hash::Hash;

/// Generic indexed hash map.
pub struct IndexedHashMap<K, V>
where
    K: Eq + Hash + Clone,
{
    /// Hash map containing the values per key.
    hashmap: HashMap<K, V>,

    /// Vector containing the keys in order of insert.
    keys: Vec<K>,
}

impl<K, V> IndexedHashMap<K, V>
where
    K: Eq + Hash + Clone,
{
    /// Creates a new indexed hash map.
    pub fn new() -> Self {
        Self {
            hashmap: HashMap::new(),
            keys: Vec::new(),
        }
    }

    /// Retrieves a specific key and value pair by index.
    pub fn get_key_value_by_index(&self, value_index: usize) -> Option<(&K, &V)> {
        match self.keys.get(value_index) {
            Some(key) => self.hashmap.get_key_value(key),
            None => None,
        }
    }

    /// Retrieves a specific key and value pair by key.
    pub fn get_key_value_by_key(&self, key: &K) -> Option<(&K, &V)> {
        self.hashmap.get_key_value(key)
    }

    /// Retrieves a specific value by index.
    pub fn get_value_by_index(&self, value_index: usize) -> Option<&V> {
        match self.keys.get(value_index) {
            Some(key) => self.hashmap.get(key),
            None => None,
        }
    }

    /// Retrieves a specific value by key.
    pub fn get_value_by_key(&self, key: &K) -> Option<&V> {
        self.hashmap.get(key)
    }

    /// Inserts a key value pair.
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if !self.hashmap.contains_key(&key) {
            self.keys.push(key.clone());
        }
        self.hashmap.insert(key, value)
    }

    /// Retrieves the number of values.
    pub fn len(&self) -> usize {
        self.hashmap.len()
    }
}
