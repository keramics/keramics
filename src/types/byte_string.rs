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

/// String of 8-bit values.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct ByteString {
    /// Elements.
    pub elements: Vec<u8>,
}

impl ByteString {
    /// Creates a new byte string.
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }

    /// Reads a byte string from a byte sequence.
    pub fn from_bytes(data: &[u8]) -> Self {
        let data_size: usize = data.len();
        let mut elements: Vec<u8> = Vec::new();

        for data_offset in 0..data_size {
            let value_8bit = data[data_offset];
            if value_8bit == 0 {
                break;
            }
            elements.push(value_8bit);
        }
        Self { elements: elements }
    }

    /// Retrieves the string representation of a byte string.
    pub fn to_string(&self) -> String {
        // TODO: add code page support
        String::from_utf8(self.elements.to_vec()).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_bytes() {
        let test_data: [u8; 12] = [
            0x41, 0x53, 0x43, 0x49, 0x49, 0x20, 0x73, 0x74, 0x72, 0x69, 0x6e, 0x67,
        ];

        let byte_string: ByteString = ByteString::from_bytes(&test_data);
        assert_eq!(byte_string.to_string(), "ASCII string".to_string(),);
    }
}
