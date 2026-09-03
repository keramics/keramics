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

use super::file_header::QcowFileHeader;

/// QEMU Copy-On-Write (QCOW) features.
pub struct QcowFeatures {
    /// Compatible feature flags.
    pub compatible_feature_flags: u64,

    /// Incompatible feature flags.
    pub incompatible_feature_flags: u64,
}

impl QcowFeatures {
    /// Creates new features.
    pub fn new() -> Self {
        Self {
            compatible_feature_flags: 0,
            incompatible_feature_flags: 0,
        }
    }

    /// Initializes the features.
    pub fn initialize(&mut self, file_header: &QcowFileHeader) {
        self.compatible_feature_flags = file_header.compatible_feature_flags;
        self.incompatible_feature_flags = file_header.incompatible_feature_flags;
    }

    /// Checks if there are unsupported features.
    pub fn is_unsupported(&self) -> bool {
        let supported_flags: u64 = 0x0000000000000001; // QCOW2_INCOMPAT_DIRTY

        /* TODO add support for:
            | 0x0000000000000008; // QCOW2_INCOMPAT_COMPRESSION
        */
        self.incompatible_feature_flags & !(supported_flags) != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize() {
        let mut file_header: QcowFileHeader = QcowFileHeader::new();
        file_header.compatible_feature_flags = 0x0000000000000001;
        file_header.incompatible_feature_flags = 0x0000000000000008;

        let mut test_struct: QcowFeatures = QcowFeatures::new();
        test_struct.initialize(&file_header);

        assert_eq!(test_struct.compatible_feature_flags, 0x0000000000000001);
        assert_eq!(test_struct.incompatible_feature_flags, 0x0000000000000008);
    }

    #[test]
    fn test_is_unsupported() {
        let mut test_struct: QcowFeatures = QcowFeatures::new();
        test_struct.incompatible_feature_flags |= 0x0000000000000004;

        assert!(test_struct.is_unsupported());
    }
}
