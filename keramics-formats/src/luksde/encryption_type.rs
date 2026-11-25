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

    /// Sets the encryption mode from a lowercase mode string.
    fn parse_encryption_mode(&mut self, mode_string: &str) {
        match mode_string.split_once('-') {
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
                self.chaining_mode = mode_string.to_string();
                self.initialization_vector_mode = None;
                self.initialization_vector_options = None;
            }
        }
    }

    /// Sets the encryption method.
    pub(super) fn set_encryption_method(&mut self, encryption_method: &ByteString) {
        self.encryption_method = encryption_method.to_string().to_lowercase();
    }

    /// Sets the encryption mode.
    pub(super) fn set_encryption_mode(&mut self, encryption_mode: &ByteString) {
        let mode_string: String = encryption_mode.to_string().to_lowercase();
        self.parse_encryption_mode(mode_string.as_str());
    }

    /// Sets the encryption method and mode from a combined LUKS2 encryption string.
    pub(super) fn set_encryption_string(&mut self, encryption_string: &str) {
        let lowercase_string: String = encryption_string.to_lowercase();

        match lowercase_string.split_once('-') {
            Some((encryption_method, encryption_mode)) => {
                self.encryption_method = encryption_method.to_string();
                self.parse_encryption_mode(encryption_mode);
            }
            None => {
                self.encryption_method = lowercase_string;
                self.chaining_mode = String::new();
                self.initialization_vector_mode = None;
                self.initialization_vector_options = None;
            }
        }
    }
}

impl fmt::Display for LuksEncryptionType {
    /// Formats the encryption type for display.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fmt() {
        let mut encryption_type: LuksEncryptionType = LuksEncryptionType::new();
        let string: String = encryption_type.to_string();
        assert_eq!(string, "N/A (not set)");

        encryption_type.set_encryption_string("aes-xts-plain64");
        encryption_type.key_size = 64;
        let string: String = encryption_type.to_string();
        assert_eq!(string, "AES-512-XTS");

        encryption_type.set_encryption_string("aes-cbc-essiv:sha256");
        encryption_type.key_size = 32;
        let string: String = encryption_type.to_string();
        assert_eq!(string, "AES-256-CBC");
    }

    #[test]
    fn test_set_encryption_mode() {
        let mut encryption_type: LuksEncryptionType = LuksEncryptionType::new();

        let encryption_mode: ByteString = ByteString::from("XTS-PLAIN64");
        encryption_type.set_encryption_mode(&encryption_mode);
        assert_eq!(encryption_type.chaining_mode, "xts");
        let initialization_vector_mode: Option<&str> =
            encryption_type.initialization_vector_mode.as_deref();
        assert_eq!(initialization_vector_mode, Some("plain64"));
        assert_eq!(encryption_type.initialization_vector_options, None);

        let encryption_mode: ByteString = ByteString::from("ECB");
        encryption_type.set_encryption_mode(&encryption_mode);
        assert_eq!(encryption_type.chaining_mode, "ecb");
        assert_eq!(encryption_type.initialization_vector_mode, None);
        assert_eq!(encryption_type.initialization_vector_options, None);
    }

    #[test]
    fn test_set_encryption_string() {
        let mut encryption_type: LuksEncryptionType = LuksEncryptionType::new();

        encryption_type.set_encryption_string("aes-xts-plain64");
        assert_eq!(encryption_type.encryption_method, "aes");
        assert_eq!(encryption_type.chaining_mode, "xts");
        let initialization_vector_mode: Option<&str> =
            encryption_type.initialization_vector_mode.as_deref();
        assert_eq!(initialization_vector_mode, Some("plain64"));
        assert_eq!(encryption_type.initialization_vector_options, None);

        encryption_type.set_encryption_string("aes-cbc-essiv:sha256");
        assert_eq!(encryption_type.encryption_method, "aes");
        assert_eq!(encryption_type.chaining_mode, "cbc");
        let initialization_vector_mode: Option<&str> =
            encryption_type.initialization_vector_mode.as_deref();
        assert_eq!(initialization_vector_mode, Some("essiv"));
        let initialization_vector_options: Option<&str> =
            encryption_type.initialization_vector_options.as_deref();
        assert_eq!(initialization_vector_options, Some("sha256"));

        // Test with encryption string without a mode separator.
        encryption_type.set_encryption_string("AES");
        assert_eq!(encryption_type.encryption_method, "aes");
        assert_eq!(encryption_type.chaining_mode, "");
        assert_eq!(encryption_type.initialization_vector_mode, None);
        assert_eq!(encryption_type.initialization_vector_options, None);
    }
}
