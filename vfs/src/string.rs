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

use std::ffi::OsString;

use types::{ByteString, Ucs2String};

/// Virtual File System (VFS) string.
#[derive(Clone, PartialEq)]
pub enum VfsString {
    Byte(ByteString),
    Empty,
    OsString(OsString),
    String(String),
    Ucs2(Ucs2String),
}

impl VfsString {
    /// Retrieves the string representation.
    pub fn to_string(&self) -> String {
        match self {
            VfsString::Byte(byte_string) => byte_string.to_string(),
            VfsString::Empty => String::new(),
            // TODO: change to_string_lossy to a non-lossy conversion
            VfsString::OsString(os_string) => os_string.to_string_lossy().to_string(),
            VfsString::String(string) => string.clone(),
            VfsString::Ucs2(ucs2_string) => ucs2_string.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: add tests
}
