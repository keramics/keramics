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

    /// Determines if the byte string is empty.
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Retrieves the length (or size) of the byte string.
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Reads the byte string from a buffer.
    pub fn read_data(&mut self, data: &[u8]) {
        let slice: &[u8] = match data.iter().position(|value| *value == 0) {
            Some(data_index) => &data[0..data_index],
            None => &data,
        };
        self.elements.extend_from_slice(&slice);
    }

    /// Converts the byte string to a `String`.
    pub fn to_string(&self) -> String {
        // TODO: add support for encoding.
        String::from_utf8(self.elements.to_vec()).unwrap()
    }
}

impl From<&[u8]> for ByteString {
    /// Converts a [`&[u8]`] into a [`ByteString`]
    fn from(slice: &[u8]) -> Self {
        let elements: &[u8] = match slice.iter().position(|value| *value == 0) {
            Some(slice_index) => &slice[0..slice_index],
            None => &slice,
        };
        Self {
            elements: Vec::from(elements),
        }
    }
}

impl From<&str> for ByteString {
    /// Converts a [`&str`] into a [`ByteString`]
    #[inline(always)]
    fn from(string: &str) -> Self {
        Self::from(string.as_bytes())
    }
}

impl From<&String> for ByteString {
    /// Converts a [`&String`] into a [`ByteString`]
    #[inline(always)]
    fn from(string: &String) -> Self {
        Self::from(string.as_str().as_bytes())
    }
}

impl From<&Vec<u8>> for ByteString {
    /// Converts a [`&Vec<u8>`] into a [`ByteString`]
    #[inline(always)]
    fn from(vector: &Vec<u8>) -> Self {
        Self::from(vector.as_slice())
    }
}

impl PartialEq<&[u8]> for ByteString {
    /// Detemines if a [`ByteString`] is equal to a [`&[u8]`]
    #[inline(always)]
    fn eq(&self, slice: &&[u8]) -> bool {
        self.elements == *slice
    }
}

impl PartialEq<&str> for ByteString {
    /// Detemines if a [`ByteString`] is equal to a [`&str`]
    #[inline(always)]
    fn eq(&self, string: &&str) -> bool {
        self.elements == *string.as_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: add test for elements

    #[test]
    fn test_is_empty() {
        let byte_string: ByteString = ByteString::new();
        assert!(byte_string.is_empty());

        let byte_string: ByteString = ByteString {
            elements: vec![
                0x41, 0x53, 0x43, 0x49, 0x49, 0x20, 0x73, 0x74, 0x72, 0x69, 0x6e, 0x67,
            ],
        };
        assert!(!byte_string.is_empty());
    }

    #[test]
    fn test_len() {
        let byte_string: ByteString = ByteString::new();
        assert_eq!(byte_string.len(), 0);

        let byte_string: ByteString = ByteString {
            elements: vec![
                0x41, 0x53, 0x43, 0x49, 0x49, 0x20, 0x73, 0x74, 0x72, 0x69, 0x6e, 0x67,
            ],
        };
        assert_eq!(byte_string.len(), 12);
    }

    #[test]
    fn test_read_data() {
        let mut byte_string: ByteString = ByteString::new();
        assert_eq!(byte_string.len(), 0);

        let test_data: [u8; 14] = [
            0x41, 0x53, 0x43, 0x49, 0x49, 0x20, 0x73, 0x74, 0x72, 0x69, 0x6e, 0x67, 0x00, 0x00,
        ];
        byte_string.read_data(&test_data);
        assert_eq!(byte_string.len(), 12);
    }

    // TODO: add test for to_string

    #[test]
    fn test_from_u8_slice() {
        let test_data: [u8; 14] = [
            0x41, 0x53, 0x43, 0x49, 0x49, 0x20, 0x73, 0x74, 0x72, 0x69, 0x6e, 0x67, 0x00, 0x00,
        ];
        let byte_string: ByteString = ByteString::from(test_data.as_slice());

        let expected_elements: Vec<u8> = vec![
            0x41, 0x53, 0x43, 0x49, 0x49, 0x20, 0x73, 0x74, 0x72, 0x69, 0x6e, 0x67,
        ];
        assert_eq!(byte_string.elements, expected_elements);
    }

    #[test]
    fn test_from_str() {
        let byte_string: ByteString = ByteString::from("ASCII string");

        let expected_elements: Vec<u8> = vec![
            0x41, 0x53, 0x43, 0x49, 0x49, 0x20, 0x73, 0x74, 0x72, 0x69, 0x6e, 0x67,
        ];
        assert_eq!(byte_string.elements, expected_elements);
    }

    #[test]
    fn test_from_string() {
        let test_string: String = String::from("ASCII string");
        let byte_string: ByteString = ByteString::from(&test_string);

        let expected_elements: Vec<u8> = vec![
            0x41, 0x53, 0x43, 0x49, 0x49, 0x20, 0x73, 0x74, 0x72, 0x69, 0x6e, 0x67,
        ];
        assert_eq!(byte_string.elements, expected_elements);
    }

    #[test]
    fn test_from_vector() {
        let test_vector: Vec<u8> = vec![
            0x41, 0x53, 0x43, 0x49, 0x49, 0x20, 0x73, 0x74, 0x72, 0x69, 0x6e, 0x67, 0x00, 0x00,
        ];
        let byte_string: ByteString = ByteString::from(&test_vector);

        let expected_elements: Vec<u8> = vec![
            0x41, 0x53, 0x43, 0x49, 0x49, 0x20, 0x73, 0x74, 0x72, 0x69, 0x6e, 0x67,
        ];
        assert_eq!(byte_string.elements, expected_elements);
    }

    // TODO: add tests for PartialEq
}
