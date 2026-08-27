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

use std::cmp::min;

use keramics_hashes::DigestHashContext;

/// Linux Unified Key Setup (LUKS) Disk Encryption diffuser.
pub struct LuksDiffuser<T: DigestHashContext, const HASH_SIZE: usize> {
    /// Digest hash context.
    digest_context: T,
}

impl<T: DigestHashContext, const HASH_SIZE: usize> LuksDiffuser<T, HASH_SIZE> {
    /// Creates a new diffuer context.
    pub fn new() -> Self {
        Self {
            digest_context: T::new(),
        }
    }

    /// Diffuses data.
    fn diffuse(&mut self, data: &mut [u8]) {
        let mut block_index: u32 = 0;
        let mut data_offset: usize = 0;
        let data_size: usize = data.len();

        while data_offset < data_size {
            let data_end_offset: usize = min(data_offset + HASH_SIZE, data_size);

            self.digest_context.update(&block_index.to_be_bytes());
            self.digest_context
                .update(&data[data_offset..data_end_offset]);

            let digest_hash: Vec<u8> = self.digest_context.finalize();

            let read_size: usize = data_end_offset - data_offset;
            data[data_offset..data_end_offset].copy_from_slice(&digest_hash[0..read_size]);

            data_offset = data_end_offset;
            block_index += 1;
        }
    }

    /// Merges split key data.
    pub fn merge(&mut self, number_of_stripes: u32, split_data: &[u8], data: &mut [u8]) {
        let mut split_data_offset: usize = 0;

        for _ in 0..(number_of_stripes - 1) {
            for byte_value in data.iter_mut() {
                *byte_value ^= split_data[split_data_offset];
                split_data_offset += 1;
            }
            self.diffuse(data);
        }
        for byte_value in data.iter_mut() {
            *byte_value ^= split_data[split_data_offset];
            split_data_offset += 1;
        }
    }
}
