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

use keramics_checksums::Fletcher64Context;

/// Apple File System (APFS) object checksum.
pub struct ApfsObjectChecksum {}

impl ApfsObjectChecksum {
    /// Calculates the checksum of a buffer.
    pub fn calculate(data: &[u8]) -> u64 {
        let mut fletcher64_context: Fletcher64Context = Fletcher64Context::new(0);
        fletcher64_context.update(data);
        let fletcher64_checksum: u64 = fletcher64_context.finalize();

        let fletcher64_lower_32bit: u64 = fletcher64_checksum & 0xffffffff;
        let fletcher64_upper_32bit: u64 = fletcher64_checksum >> 32;
        let checksum_lower_32bit: u64 =
            0xffffffff - ((fletcher64_lower_32bit + fletcher64_upper_32bit) % 0xffffffff);
        let checksum_upper_32bit: u64 =
            0xffffffff - ((fletcher64_lower_32bit + checksum_lower_32bit) % 0xffffffff);

        (checksum_upper_32bit << 32) | checksum_lower_32bit
    }
}
