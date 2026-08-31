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

/// Apple File System (APFS) B-tree entry.
#[derive(Clone)]
pub struct ApfsBtreeEntry {
    /// Key data offset.
    pub key_data_offset: usize,

    /// Key data size.
    pub key_data_size: usize,

    /// Value data offset.
    pub value_data_offset: usize,

    /// Value data size.
    pub value_data_size: usize,
}

impl ApfsBtreeEntry {
    /// Creates a new B-tree entry.
    pub fn new() -> Self {
        Self {
            key_data_offset: 0,
            key_data_size: 0,
            value_data_offset: 0,
            value_data_size: 0,
        }
    }
}
