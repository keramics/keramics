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

use keramics_core::ErrorTrace;
use keramics_encodings::CharacterEncoding;
use keramics_types::ByteString;

/// X File System (XFS) attribute.
pub enum XfsAttribute {
    InlineData(Vec<u8>),
}

impl XfsAttribute {
    /// Reads the attributes entry name from a buffer.
    pub fn read_name(
        character_encoding: &CharacterEncoding,
        attribute_flags: u8,
        data: &[u8],
    ) -> ByteString {
        let mut name: ByteString = ByteString::new_with_encoding(character_encoding);

        let name_prefix: &str = match attribute_flags & 0x7e {
            0x00 => "user.",
            0x02 => "trusted.",
            0x04 => "secure.",
            _ => "",
        };
        name.read_data(name_prefix.as_bytes());
        name.read_data(data);

        name
    }
}
