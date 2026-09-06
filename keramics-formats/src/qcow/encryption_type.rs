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

/// QEMU Copy-On-Write (QCOW) encryption type.
#[derive(Clone, Debug, PartialEq)]
pub struct QcowEncryptionType {
    /// Method.
    pub(super) method: u32,
}

impl QcowEncryptionType {
    /// Creates a new encryption type.
    pub(crate) fn new(method: u32) -> Self {
        Self { method }
    }
}

impl fmt::Display for QcowEncryptionType {
    /// Formats encryption type for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self.method {
            0 => write!(formatter, "N/A (not set)"),
            1 => write!(formatter, "AES-128-CBC"),
            2 => write!(formatter, "Linux Unified Key Setup (LUKS)"),
            _ => write!(formatter, "N/A (unsupported)"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_with_not_set() {
        let encryption_type: QcowEncryptionType = QcowEncryptionType::new(0);
        assert_eq!(encryption_type.to_string(), "N/A (not set)");
    }

    #[test]
    fn test_display_with_aes_cbc() {
        let encryption_type: QcowEncryptionType = QcowEncryptionType::new(1);
        assert_eq!(encryption_type.to_string(), "AES-128-CBC");
    }

    #[test]
    fn test_display_with_luks() {
        let encryption_type: QcowEncryptionType = QcowEncryptionType::new(2);
        assert_eq!(
            encryption_type.to_string(),
            "Linux Unified Key Setup (LUKS)"
        );
    }

    #[test]
    fn test_display_with_unsupported_method() {
        let encryption_type: QcowEncryptionType = QcowEncryptionType::new(0xffffffff);
        assert_eq!(encryption_type.to_string(), "N/A (unsupported)");
    }
}
