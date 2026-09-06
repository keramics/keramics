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

/// BDE file system signature.
pub(crate) const BDE_FILE_SYSTEM_SIGNATURE: &[u8] = b"-FVE-FS-";

/// BDE boot entry point Windows Vista boot record.
pub(super) const BDE_BOOT_ENTRY_POINT_VISTA: &[u8] = &[0xeb, 0x52, 0x90];

/// BDE identifier.
pub(crate) const BDE_IDENTIFIER: &[u8] = &[
    0x3b, 0xd6, 0x67, 0x49, 0x29, 0x2e, 0xd8, 0x4a, 0x83, 0x99, 0xf6, 0xa3, 0x39, 0xe3, 0xd0, 0x01,
];

/// BDE Used Disk Space Only encryption identifier.
pub(crate) const BDE_USED_DISK_SPACE_ONLY_IDENTIFIER: &[u8] = &[
    0x3b, 0x4d, 0xa8, 0x92, 0x80, 0xdd, 0x0e, 0x4d, 0x9e, 0x4e, 0xb1, 0xe3, 0x28, 0x4e, 0xae, 0xd8,
];
