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

/// UDIF file footer signature: "koly".
pub(super) const UDIF_FILE_FOOTER_SIGNATURE: [u8; 4] = [0x6b, 0x6f, 0x6c, 0x79];

/// UDIF block table header signature: "mish".
pub(super) const UDIF_BLOCK_TABLE_HEADER_SIGNATURE: [u8; 4] = [0x6d, 0x69, 0x73, 0x68];
