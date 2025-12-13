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

/// Descriptor file header signature.
pub(crate) const VMDK_DESCRIPTOR_FILE_HEADER_SIGNATURE: &[u8] = b"# Disk DescriptorFile";

/// Sparse Copy-On-Write Disk (COWD) file header signature.
pub(super) const VMDK_SPARSE_COWD_FILE_HEADER_SIGNATURE: &[u8] = b"COWD";

/// VMDK sparse file header signature.
pub(crate) const VMDK_SPARSE_FILE_HEADER_SIGNATURE: &[u8] = b"KDMV";

/// VMDK sparse file flags.
pub const VMDK_SPARSE_FILE_FLAG_USE_SECONDARY_GRAIN_DIRECTORY: u32 = 0x00000002;
pub const VMDK_SPARSE_FILE_FLAG_HAS_GRAIN_COMPRESSION: u32 = 0x00010000;
pub const VMDK_SPARSE_FILE_FLAG_HAS_DATA_MARKERS: u32 = 0x00020000;
