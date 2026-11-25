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

/// Linux Unified Key Setup (LUKS) version 2 metadata segment object.
#[derive(Clone, Debug, PartialEq)]
pub struct LuksMetadataSegment {
    /// Segment identifier.
    pub identifier: String,

    /// Segment type (for example, "crypt" or "linear").
    pub segment_type: String,

    /// Segment offset in bytes.
    pub offset: u64,

    /// Segment size in bytes (`None` for "dynamic").
    pub size: Option<u64>,

    /// Initialization vector tweak value.
    pub initialization_vector_tweak: u64,

    /// Segment encryption specification.
    pub encryption: String,

    /// Sector size in bytes.
    pub sector_size: u16,
}

impl LuksMetadataSegment {
    /// Creates a new metadata segment object.
    pub fn new(identifier: String) -> Self {
        Self {
            identifier,
            segment_type: String::new(),
            offset: 0,
            size: None,
            initialization_vector_tweak: 0,
            encryption: String::new(),
            sector_size: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let identifier: String = String::from("0");
        let segment: LuksMetadataSegment = LuksMetadataSegment::new(identifier);

        assert_eq!(segment.identifier, "0");
        assert!(segment.segment_type.is_empty());
        assert_eq!(segment.offset, 0);
        assert_eq!(segment.size, None);
        assert_eq!(segment.initialization_vector_tweak, 0);
        assert!(segment.encryption.is_empty());
        assert_eq!(segment.sector_size, 0);
    }
}
