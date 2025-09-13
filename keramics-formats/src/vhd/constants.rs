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

/// VHD fixed disk type.
pub(super) const VHD_DISK_TYPE_FIXED: u32 = 2;

/// VHD dynamic disk type.
pub(super) const VHD_DISK_TYPE_DYNAMIC: u32 = 3;

/// VHD differential disk type.
pub(super) const VHD_DISK_TYPE_DIFFERENTIAL: u32 = 4;

/// VHD disk types.
pub(super) const VHD_DISK_TYPES: &[u32] = &[
    VHD_DISK_TYPE_FIXED,
    VHD_DISK_TYPE_DYNAMIC,
    VHD_DISK_TYPE_DIFFERENTIAL,
];

/// VHD dynamic disk header signature: "cxsparse".
pub(super) const VHD_DYNAMIC_DISK_HEADER_SIGNATURE: [u8; 8] =
    [0x63, 0x78, 0x73, 0x70, 0x61, 0x72, 0x73, 0x65];

/// VHD file footer signature: "conectix".
pub(super) const VHD_FILE_FOOTER_SIGNATURE: [u8; 8] =
    [0x63, 0x6f, 0x6e, 0x65, 0x63, 0x74, 0x69, 0x78];
