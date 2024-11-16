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

/// APM partition map signature.
pub(super) const APM_PARTITION_MAP_SIGNATURE: [u8; 2] = [0x50, 0x4d];

/// APM partition map partition type.
pub(super) const APM_PARTITION_MAP_TYPE: [u8; 19] = [
    0x41, 0x70, 0x70, 0x6c, 0x65, 0x5f, 0x70, 0x61, 0x72, 0x74, 0x69, 0x74, 0x69, 0x6f, 0x6e, 0x5f,
    0x6d, 0x61, 0x70,
]; // "Apple_partition_map"
