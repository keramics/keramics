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

/// ExFAT boot signature.
pub(super) const EXFAT_BOOT_SIGNATURE: &[u8] = b"\x55\xaa";

/// Largest cluster block number.
pub(super) const EXFAT_LARGEST_CLUSTER_BLOCK_NUMBER: u32 = 0xfffffff0;

/// Directory file attribute flag.
pub(super) const EXFAT_FILE_ATTRIBUTE_FLAG_DIRECTORY: u16 = 0x0010;
