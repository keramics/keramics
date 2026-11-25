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

/// Linux Unified Key Setup (LUKS) version 2 metadata keyslot object.
#[derive(Clone, Debug, PartialEq)]
pub struct LuksMetadataKeySlot {
    /// Keyslot identifier.
    pub identifier: String,

    /// Keyslot type (for example, "luks2" or "reencrypt").
    pub keyslot_type: String,

    /// Master key size in bytes.
    pub key_size: usize,

    /// Optional priority (0 = ignore, 1 = normal, 2 = high).
    pub priority: Option<i64>,

    /// Anti-forensic diffuser type (for example, "luks1").
    pub anti_forensic_type: String,

    /// Anti-forensic diffuser hashing method.
    pub anti_forensic_hashing_method: String,

    /// Anti-forensic diffuser number of stripes.
    pub anti_forensic_number_of_stripes: u32,

    /// Keyslot area type (for example, "raw").
    pub area_type: String,

    /// Keyslot area offset in bytes.
    pub area_offset: u64,

    /// Keyslot area size in bytes.
    pub area_size: u64,

    /// Keyslot area encryption specification.
    pub area_encryption: String,

    /// Keyslot area key size in bytes.
    pub area_key_size: usize,

    /// Key derivation function type (for example, "pbkdf2", "argon2i", "argon2id").
    pub key_derivation_type: String,

    /// PBKDF2 hashing method.
    pub key_derivation_hashing_method: String,

    /// PBKDF2 number of iterations.
    pub key_derivation_number_of_iterations: u32,

    /// Argon2 time cost (number of iterations).
    pub key_derivation_time: u32,

    /// Argon2 memory cost in KiB.
    pub key_derivation_memory: u32,

    /// Argon2 parallelism (number of threads/lanes).
    pub key_derivation_parallelism: u32,

    /// Key derivation salt.
    pub key_derivation_salt: Vec<u8>,
}

impl LuksMetadataKeySlot {
    /// Creates a new metadata keyslot object.
    pub fn new(identifier: String) -> Self {
        Self {
            identifier,
            keyslot_type: String::new(),
            key_size: 0,
            priority: None,
            anti_forensic_type: String::new(),
            anti_forensic_hashing_method: String::new(),
            anti_forensic_number_of_stripes: 0,
            area_type: String::new(),
            area_offset: 0,
            area_size: 0,
            area_encryption: String::new(),
            area_key_size: 0,
            key_derivation_type: String::new(),
            key_derivation_hashing_method: String::new(),
            key_derivation_number_of_iterations: 0,
            key_derivation_time: 0,
            key_derivation_memory: 0,
            key_derivation_parallelism: 0,
            key_derivation_salt: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let identifier: String = String::from("0");
        let key_slot: LuksMetadataKeySlot = LuksMetadataKeySlot::new(identifier);

        let expected_key_slot: LuksMetadataKeySlot = LuksMetadataKeySlot {
            identifier: String::from("0"),
            keyslot_type: String::new(),
            key_size: 0,
            priority: None,
            anti_forensic_type: String::new(),
            anti_forensic_hashing_method: String::new(),
            anti_forensic_number_of_stripes: 0,
            area_type: String::new(),
            area_offset: 0,
            area_size: 0,
            area_encryption: String::new(),
            area_key_size: 0,
            key_derivation_type: String::new(),
            key_derivation_hashing_method: String::new(),
            key_derivation_number_of_iterations: 0,
            key_derivation_time: 0,
            key_derivation_memory: 0,
            key_derivation_parallelism: 0,
            key_derivation_salt: Vec::new(),
        };
        assert_eq!(key_slot, expected_key_slot);
    }
}
