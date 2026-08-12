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

/// EWF-E01 and EWF-s01 file header signature
pub(crate) const EWF_FILE_HEADER_SIGNATURE: &[u8] = b"EVF\x09\x0d\x0a\xff\x00";

/// EWF-L01 file header signature
pub(crate) const EWF_L01_FILE_HEADER_SIGNATURE: &[u8] = b"LVF\x09\x0d\x0a\xff\x00";

/// EWF data section type
pub(super) const EWF_SECTION_TYPE_DATA: &[u8] = b"data\0\0\0\0\0\0\0\0\0\0\0\0";

/// EWF digest section type
pub(super) const EWF_SECTION_TYPE_DIGEST: &[u8] = b"digest\0\0\0\0\0\0\0\0\0\0";

/// EWF disk section type
pub(super) const EWF_SECTION_TYPE_DISK: &[u8] = b"disk\0\0\0\0\0\0\0\0\0\0\0\0";

/// EWF done section type
pub(super) const EWF_SECTION_TYPE_DONE: &[u8] = b"done\0\0\0\0\0\0\0\0\0\0\0\0";

/// EWF error2 section type
pub(super) const EWF_SECTION_TYPE_ERROR2: &[u8] = b"error2\0\0\0\0\0\0\0\0\0\0";

/// EWF hash section type
pub(super) const EWF_SECTION_TYPE_HASH: &[u8] = b"hash\0\0\0\0\0\0\0\0\0\0\0\0";

/// EWF header section type
pub(super) const EWF_SECTION_TYPE_HEADER: &[u8] = b"header\0\0\0\0\0\0\0\0\0\0";

/// EWF header2 section type
pub(super) const EWF_SECTION_TYPE_HEADER2: &[u8] = b"header2\0\0\0\0\0\0\0\0\0";

/// EWF ltree section type
pub(super) const EWF_SECTION_TYPE_LTREE: &[u8] = b"ltree\0\0\0\0\0\0\0\0\0\0\0";

// TODO: ltypes

/// EWF next section type
pub(super) const EWF_SECTION_TYPE_NEXT: &[u8] = b"next\0\0\0\0\0\0\0\0\0\0\0\0";

/// EWF sectors section type
pub(super) const EWF_SECTION_TYPE_SECTORS: &[u8] = b"sectors\0\0\0\0\0\0\0\0\0";

// TODO: session

/// EWF table section type
pub(super) const EWF_SECTION_TYPE_TABLE: &[u8] = b"table\0\0\0\0\0\0\0\0\0\0\0";

/// EWF table section type
pub(super) const EWF_SECTION_TYPE_TABLE2: &[u8] = b"table2\0\0\0\0\0\0\0\0\0\0";

/// EWF volume section type
pub(super) const EWF_SECTION_TYPE_VOLUME: &[u8] = b"volume\0\0\0\0\0\0\0\0\0\0";

// TODO: xhash
// TODO: xheader
