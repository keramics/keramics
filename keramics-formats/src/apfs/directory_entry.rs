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

use keramics_types::ByteString;

use super::directory_record::ApfsDirectoryRecord;

/// Apple File System (APFS) directory entry.
#[derive(Clone)]
pub struct ApfsDirectoryEntry {
    /// Name.
    pub name: Option<ByteString>,

    /// Directory entry record.
    record: ApfsDirectoryRecord,
}

impl ApfsDirectoryEntry {
    /// Creates a new directory entry.
    pub fn new(record: ApfsDirectoryRecord) -> Self {
        Self { name: None, record }
    }

    /// Retrieves the identifier.
    pub fn get_identifier(&self) -> u64 {
        self.record.object_identifier
    }
}
