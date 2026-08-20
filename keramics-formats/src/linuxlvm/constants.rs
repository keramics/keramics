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

/// Linux Logical Volume Manager (LVM) physical volume label signature.
pub(crate) const LINUX_LVM_PHYSICAL_VOLUME_SIGNATURE: &[u8] = b"LABELONE";

/// Linux Logical Volume Manager (LVM) physical volume label type indicator.
pub(super) const LINUX_LVM_PHYSICAL_VOLUME_TYPE_INDICATOR: &[u8] = b"LVM2 001";

/// Linux Logical Volume Manager (LVM) metadata area signature,
pub(super) const LINUX_LVM_METADATA_AREA_SIGNATURE: &[u8] = b" LVM2 x[5A%r0N*>";
