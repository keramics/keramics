/* Copyright 2024-2025 Joachim Metz <joachim.metz@gmail.com>
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

/// FAT boot signature: "\x55\xaa".
pub(super) const FAT_BOOT_SIGNATURE: [u8; 2] = [0x55, 0xaa];

/// Support bytes per sector values.
pub(super) const FAT_SUPPORTED_BYTES_PER_SECTOR: [u16; 4] = [512, 1024, 2048, 4096];

/// Support sector per cluster block values.
pub(super) const FAT_SUPPORTED_SECTORS_PER_CLUSTER_BLOCK: [u8; 8] = [1, 2, 4, 8, 16, 32, 64, 128];

/// Volume label file attribute flag.
pub(super) const FAT_FILE_ATTRIBUTE_FLAG_VOLUME_LABEL: u8 = 0x08;

/// Directory file attribute flag.
pub(super) const FAT_FILE_ATTRIBUTE_FLAG_DIRECTORY: u8 = 0x10;
