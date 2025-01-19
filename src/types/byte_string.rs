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

/// String of 8-bit elements.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ByteString {
    /// Elements.
    pub elements: Vec<u8>,
}

// TODO: add support for encoding.
impl ByteString {
    /// Creates a new byte string.
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }

    /// Creates a new byte string from a byte sequence.
    pub fn from_bytes(data: &[u8]) -> Self {
        let mut elements: Vec<u8> = Vec::new();
        ByteString::read_elements(&mut elements, data);

        Self { elements: elements }
    }

    /// Creates a new byte string from a string.
    pub fn from_string(string: &str) -> Self {
        // TODO: add support for encoding.
        ByteString::from_bytes(string.as_bytes())
    }

    /// Determines if the byte string is empty.
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Retrieves the length (or size) of the byte string.
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Reads byte string elements from a byte sequence.
    pub fn read_elements(elements: &mut Vec<u8>, data: &[u8]) {
        let data_size: usize = data.len();

        for data_offset in 0..data_size {
            let value_8bit = data[data_offset];
            if value_8bit == 0 {
                break;
            }
            elements.push(value_8bit);
        }
    }

    /// Retrieves the string representation of the byte string.
    pub fn to_string(&self) -> String {
        // TODO: add support for encoding.
        String::from_utf8(self.elements.to_vec()).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_bytes() {
        let test_data: [u8; 14] = [
            0x41, 0x53, 0x43, 0x49, 0x49, 0x20, 0x73, 0x74, 0x72, 0x69, 0x6e, 0x67, 0x00, 0x00,
        ];
        let byte_string: ByteString = ByteString::from_bytes(&test_data);

        let expected_elements: Vec<u8> = vec![
            0x41, 0x53, 0x43, 0x49, 0x49, 0x20, 0x73, 0x74, 0x72, 0x69, 0x6e, 0x67,
        ];
        assert_eq!(byte_string.elements, expected_elements);
    }

    #[test]
    fn test_from_string() {
        let byte_string: ByteString = ByteString::from_string("ASCII string");

        let expected_elements: Vec<u8> = vec![
            0x41, 0x53, 0x43, 0x49, 0x49, 0x20, 0x73, 0x74, 0x72, 0x69, 0x6e, 0x67,
        ];
        assert_eq!(byte_string.elements, expected_elements);
    }

    #[test]
    fn test_is_empty() {
        let byte_string: ByteString = ByteString::new();
        assert!(byte_string.is_empty());

        let test_data: [u8; 14] = [
            0x41, 0x53, 0x43, 0x49, 0x49, 0x20, 0x73, 0x74, 0x72, 0x69, 0x6e, 0x67, 0x00, 0x00,
        ];
        let byte_string: ByteString = ByteString::from_bytes(&test_data);
        assert!(!byte_string.is_empty());
    }

    #[test]
    fn test_len() {
        let byte_string: ByteString = ByteString::new();
        assert_eq!(byte_string.len(), 0);

        let test_data: [u8; 14] = [
            0x41, 0x53, 0x43, 0x49, 0x49, 0x20, 0x73, 0x74, 0x72, 0x69, 0x6e, 0x67, 0x00, 0x00,
        ];
        let byte_string: ByteString = ByteString::from_bytes(&test_data);
        assert_eq!(byte_string.len(), 12);
    }

    // TODO: add test for read_elements
    // TODO: add test for to_string
}
