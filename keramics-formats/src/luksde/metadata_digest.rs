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

/// Linux Unified Key Setup (LUKS) version 2 metadata digest object.
#[derive(Clone, Debug, PartialEq)]
pub struct LuksMetadataDigest {
    /// Digest identifier.
    pub identifier: String,

    /// Digest type (for example, "pbkdf2").
    pub digest_type: String,

    /// Keyslot identifiers validated by this digest.
    pub keyslots: Vec<String>,

    /// Segment identifiers validated by this digest.
    pub segments: Vec<String>,

    /// Hashing method.
    pub hashing_method: String,

    /// Number of iterations.
    pub number_of_iterations: u32,

    /// Salt.
    pub salt: Vec<u8>,

    /// Expected digest bytes.
    pub digest: Vec<u8>,
}

impl LuksMetadataDigest {
    /// Creates a new metadata digest object.
    pub fn new(identifier: String) -> Self {
        Self {
            identifier,
            digest_type: String::new(),
            keyslots: Vec::new(),
            segments: Vec::new(),
            hashing_method: String::new(),
            number_of_iterations: 0,
            salt: Vec::new(),
            digest: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let identifier: String = String::from("0");
        let digest: LuksMetadataDigest = LuksMetadataDigest::new(identifier);

        assert_eq!(digest.identifier, "0");
        assert!(digest.digest_type.is_empty());
        assert!(digest.keyslots.is_empty());
        assert!(digest.segments.is_empty());
        assert!(digest.hashing_method.is_empty());
        assert_eq!(digest.number_of_iterations, 0);
        assert!(digest.salt.is_empty());
        assert!(digest.digest.is_empty());
    }
}
