/* Copyright 2024 Joachim Metz <joachim.metz@gmail.com>
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

use crate::types::Uuid;

/// VHDX image header signature: "head".
pub(super) const VHDX_IMAGE_HEADER_SIGNATURE: [u8; 4] = [0x68, 0x65, 0x61, 0x64];

/// VHDX region table header signature: "regi".
pub(super) const VHDX_REGION_TABLE_HEADER_SIGNATURE: [u8; 4] = [0x72, 0x65, 0x67, 0x69];

/// VHDX metadata table header signature: "metadata".
pub(super) const VHDX_METADATA_TABLE_HEADER_SIGNATURE: [u8; 8] =
    [0x6d, 0x65, 0x74, 0x61, 0x64, 0x61, 0x74, 0x61];

/// VHDX parent locator type indicator: {b04aefb7-d19e-4a81-b789-25b8e9445913}
pub(super) const VHDX_PARENT_LOCATOR_TYPE_INDICATOR: [u8; 16] = [
    0xb7, 0xef, 0x4a, 0xb0, 0x9e, 0xd1, 0x81, 0x4a, 0xb7, 0x89, 0x25, 0xb8, 0xe9, 0x44, 0x59, 0x13,
];

/// VHDX block allocation table (BAT) region (type) identifier: {2dc27766-f623-4200-9d64-115e9bfd4a08}
pub(super) const VHDX_BLOCK_ALLOCATION_TABLE_REGION_IDENTIFIER: Uuid = Uuid {
    part1: 0x2dc27766,
    part2: 0xf623,
    part3: 0x4200,
    part4: 0x9d64,
    part5: 0x115e9bfd4a08,
};

/// VHDX metadata region (type) identifier: {8b7ca206-4790-4b9a-b8fe-575f050f886e}
pub(super) const VHDX_METADATA_REGION_IDENTIFIER: Uuid = Uuid {
    part1: 0x8b7ca206,
    part2: 0x4790,
    part3: 0x4b9a,
    part4: 0xb8fe,
    part5: 0x575f050f886e,
};

/// VHDX virtual disk size metadata (item) identifier: {2fa54224-cd1b-4876-b211-5dbed83bf4b8}
pub(super) const VHDX_VIRTUAL_DISK_SIZE_METADATA_IDENTIFIER: Uuid = Uuid {
    part1: 0x2fa54224,
    part2: 0xcd1b,
    part3: 0x4876,
    part4: 0xb211,
    part5: 0x5dbed83bf4b8,
};

/// VHDX logical sector size metadata (item) identifier: {8141bf1d-a96f-4709-ba47-f233a8faab5f}
pub(super) const VHDX_LOGICAL_SECTOR_SIZE_METADATA_IDENTIFIER: Uuid = Uuid {
    part1: 0x8141bf1d,
    part2: 0xa96f,
    part3: 0x4709,
    part4: 0xba47,
    part5: 0xf233a8faab5f,
};

/// VHDX parent locator metadata (item) identifier: {a8d35f2d-b30b-454d-abf7-d3d84834ab0c}
pub(super) const VHDX_PARENT_LOCATOR_METADATA_IDENTIFIER: Uuid = Uuid {
    part1: 0xa8d35f2d,
    part2: 0xb30b,
    part3: 0x454d,
    part4: 0xabf7,
    part5: 0xd3d84834ab0c,
};

/// VHDX virtual disk identifier metadata (item) identifier: {beca12ab-b2e6-4523-93ef-c309e000c746}
pub(super) const VHDX_VIRTUAL_DISK_IDENTIFIER_METADATA_IDENTIFIER: Uuid = Uuid {
    part1: 0xbeca12ab,
    part2: 0xb2e6,
    part3: 0x4523,
    part4: 0x93ef,
    part5: 0xc309e000c746,
};

/// VHDX file parameters metadata (item) identifier: {caa16737-fa36-4d43-b3b6-33f0aa44e76b}
pub(super) const VHDX_FILE_PARAMETERS_METADATA_IDENTIFIER: Uuid = Uuid {
    part1: 0xcaa16737,
    part2: 0xfa36,
    part3: 0x4d43,
    part4: 0xb3b6,
    part5: 0x33f0aa44e76b,
};

/// VHDX phsysical sector size metadata (item) identifier: {cda348c7-445d-4471-9cc9-e9885251c556}
pub(super) const VHDX_PHYSICAL_SECTOR_SIZE_METADATA_IDENTIFIER: Uuid = Uuid {
    part1: 0xcda348c7,
    part2: 0x445d,
    part3: 0x4471,
    part4: 0x9cc9,
    part5: 0xe9885251c556,
};
