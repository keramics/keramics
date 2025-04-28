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

/// NTFS file system signature: "NTFS    ".
pub(super) const NTFS_FILE_SYSTEM_SIGNATURE: [u8; 8] =
    [0x4e, 0x54, 0x46, 0x53, 0x20, 0x20, 0x20, 0x20];

/// NTFS MFT entry signature: "FILE".
pub(super) const NTFS_MFT_ENTRY_SIGNATURE: [u8; 4] = [0x46, 0x49, 0x4c, 0x45];

/// NTFS bad MFT entry signature: "BAAD".
pub(super) const NTFS_BAD_MFT_ENTRY_SIGNATURE: [u8; 4] = [0x42, 0x41, 0x41, 0x44];

/// NTFS index entry signature: "INDX".
pub(super) const NTFS_INDEX_ENTRY_SIGNATURE: [u8; 4] = [0x49, 0x4e, 0x44, 0x58];

/// NTFS $STANDARD_INFORMATION attribute type.
pub const NTFS_ATTRIBUTE_TYPE_STANDARD_INFORMATION: u32 = 0x00000010;

/// NTFS $ATTRIBUTE_LIST attribute type.
pub const NTFS_ATTRIBUTE_TYPE_ATTRIBUTE_LIST: u32 = 0x00000020;

/// NTFS $FILE_NAME attribute type.
pub const NTFS_ATTRIBUTE_TYPE_FILE_NAME: u32 = 0x00000030;

/// NTFS $OBJECT_ID attribute type.
pub const NTFS_ATTRIBUTE_TYPE_OBJECT_IDENTIFIER: u32 = 0x00000040;

/// NTFS $SECURITY_DESCRIPTOR attribute type.
pub const NTFS_ATTRIBUTE_TYPE_SECURITY_DESCRIPTOR: u32 = 0x00000050;

/// NTFS $VOLUME_NAME attribute type.
pub const NTFS_ATTRIBUTE_TYPE_VOLUME_NAME: u32 = 0x00000060;

/// NTFS $VOLUME_INFORMATION attribute type.
pub const NTFS_ATTRIBUTE_TYPE_VOLUME_INFORMATION: u32 = 0x00000070;

/// NTFS $DATA attribute type.
pub const NTFS_ATTRIBUTE_TYPE_DATA: u32 = 0x00000080;

/// NTFS $INDEX_ROOT attribute type.
pub const NTFS_ATTRIBUTE_TYPE_INDEX_ROOT: u32 = 0x00000090;

/// NTFS $INDEX_ALLOCATION attribute type.
pub const NTFS_ATTRIBUTE_TYPE_INDEX_ALLOCATION: u32 = 0x000000a0;

/// NTFS $BITMAP attribute type.
pub const NTFS_ATTRIBUTE_TYPE_BITMAP: u32 = 0x000000b0;

/// NTFS $REPARSE_POINT attribute type.
pub const NTFS_ATTRIBUTE_TYPE_REPARSE_POINT: u32 = 0x000000c0;

/// NTFS $EA_INFORMATION attribute type.
pub const NTFS_ATTRIBUTE_TYPE_EXTENDED_INFORMATION: u32 = 0x000000d0;

/// NTFS $EA attribute type.
pub const NTFS_ATTRIBUTE_TYPE_EXTENDED: u32 = 0x000000e0;

/// NTFS $PROPERTY_SET attribute type.
pub const NTFS_ATTRIBUTE_TYPE_PROPERTY_SET: u32 = 0x000000f0;

/// NTFS $LOGGED_UTILITY_STREAM attribute type.
pub const NTFS_ATTRIBUTE_TYPE_LOGGED_UTILITY_STREAM: u32 = 0x00000100;

/// NTFS volume information file ("$Volume") identifier (MFT entry number).
pub(super) const NTFS_VOLUME_INFORMATION_FILE_IDENTIFIER: u64 = 3;

/// NTFS root directory (".") identifier (MFT entry number).
pub(super) const NTFS_ROOT_DIRECTORY_IDENTIFIER: u64 = 5;

/// NTFS case folding mappings file ("$UpCase") identifier (MFT entry number).
pub(super) const NTFS_CASE_FOLDING_MAPPIINGS_FILE_IDENTIFIER: u64 = 10;

/// NTFS index value flag to indicate the node is a branch node.
pub(super) const NTFS_INDEX_VALUE_FLAG_IS_BRANCH: u32 = 0x00000001;

/// NTFS index value flag to indicate the value is the last value in node.
pub(super) const NTFS_INDEX_VALUE_FLAG_IS_LAST: u32 = 0x00000002;

/// NTFS DOS name space.
pub(super) const NTFS_NAME_SPACE_DOS: u8 = 0x02;
