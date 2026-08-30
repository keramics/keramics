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

/// X File System (XFS) directory entry.
pub struct XfsDirectoryEntry {
    /// The (absolute) inode number.
    pub inode_number: u64,

    /// The parent inode number.
    pub parent_inode_number: u64,
}

impl XfsDirectoryEntry {
    /// Creates a new directory entry.
    pub fn new(inode_number: u64, parent_inode_number: u64) -> Self {
        Self {
            inode_number,
            parent_inode_number,
        }
    }
}
