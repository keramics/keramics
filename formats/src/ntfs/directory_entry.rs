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

use types::Ucs2String;

use super::file_name_attribute::NtfsFileNameAttribute;

#[derive(Clone)]
/// New Technologies File System (NTFS) directory entry.
pub struct NtfsDirectoryEntry {
    /// File reference.
    pub file_reference: u64,

    /// File name attribute.
    pub file_name_attribute: NtfsFileNameAttribute,

    /// Short file name attribute.
    pub short_file_name_attribute: Option<NtfsFileNameAttribute>,
}

impl NtfsDirectoryEntry {
    /// Creates a new directory entry.
    pub fn new(file_reference: u64, file_name_attribute: NtfsFileNameAttribute) -> Self {
        Self {
            file_reference: file_reference,
            file_name_attribute: file_name_attribute,
            short_file_name_attribute: None,
        }
    }

    /// Rerieves the name.
    pub fn get_name(&self) -> &Ucs2String {
        &self.file_name_attribute.name
    }
}
