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

/// Extensible File Allocation Table (exFAT) checksum.
pub struct ExFatChecksum {}

impl ExFatChecksum {
    /// Calculates the checksum of a buffer.
    pub fn calculate(data: &[u8]) -> u32 {
        let mut checksum: u32 = 0;

        for byte_value in data.iter() {
            checksum = if (checksum & 1) != 0 { 0x80000000 } else { 0 }
                + (checksum >> 1)
                + (*byte_value as u32);
        }
        checksum
    }
}
