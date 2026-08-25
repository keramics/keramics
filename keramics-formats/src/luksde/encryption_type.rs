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

use std::fmt;

use keramics_types::ByteString;

/// Linux Unified Key Setup (LUKS) Disk Encryption encryption type.
#[derive(Clone, Debug, PartialEq)]
pub struct LuksEncryptionType {
    /// Encryption method.
    pub(super) encryption_method: String,

    /// Chaining mode.
    pub(super) chaining_mode: String,

    /// Key size.
    pub(super) key_size: usize,

    /// Initialization vector mode.
    pub(super) initialization_vector_mode: Option<String>,

    /// Initialization vector options.
    pub(super) initialization_vector_options: Option<String>,
}

impl LuksEncryptionType {
    /// Creates a new encryption type.
    pub(super) fn new() -> Self {
        Self {
            encryption_method: String::new(),
            chaining_mode: String::new(),
            key_size: 0,
            initialization_vector_mode: None,
            initialization_vector_options: None,
        }
    }

    /// Sets the encryption method.
    pub(super) fn set_encryption_method(&mut self, encryption_method: &ByteString) {
        self.encryption_method = encryption_method.to_string().to_lowercase()
    }

    /// Sets the encryption mode.
    pub(super) fn set_encryption_mode(&mut self, encryption_mode: &ByteString) {
        let mode_string: String = encryption_mode.to_string().to_lowercase();

        match mode_string.as_str().split_once('-') {
            Some((chaining_mode, remainder)) => {
                self.chaining_mode = chaining_mode.to_string();

                match remainder.split_once(':') {
                    Some((mode, options)) => {
                        self.initialization_vector_mode = Some(mode.to_string());
                        self.initialization_vector_options = Some(options.to_string());
                    }
                    None => {
                        self.initialization_vector_mode = Some(remainder.to_string());
                        self.initialization_vector_options = None;
                    }
                }
            }
            None => {
                self.chaining_mode = mode_string;
                self.initialization_vector_mode = None;
                self.initialization_vector_options = None;
            }
        }
    }
}

impl fmt::Display for LuksEncryptionType {
    /// Formats encryption type for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        if self.encryption_method.is_empty() {
            write!(formatter, "N/A (not set)")
        } else {
            write!(
                formatter,
                "{}-{}-{}",
                self.encryption_method.to_uppercase(),
                self.key_size * 8,
                self.chaining_mode.to_uppercase()
            )
        }
    }
}
