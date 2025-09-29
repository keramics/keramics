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

/// Lines iterator for header and ltree data.
pub struct EwfLinesIterator<'a> {
    /// Data.
    data: &'a Vec<u8>,

    /// Data size.
    data_size: usize,

    /// Data offset.
    data_offset: usize,
}

impl<'a> EwfLinesIterator<'a> {
    /// Creates a new iterator.
    pub fn new(data: &'a Vec<u8>) -> Self {
        Self {
            data: data,
            data_size: data.len(),
            data_offset: 0,
        }
    }
}

impl<'a> Iterator for EwfLinesIterator<'a> {
    type Item = &'a [u8];

    /// Retrieves the next line.
    fn next(&mut self) -> Option<Self::Item> {
        if self.data[self.data_offset] == 0 {
            return None;
        }
        let start_offset: usize = self.data_offset;

        while self.data_offset < self.data_size {
            let byte: u8 = self.data[self.data_offset];

            self.data_offset += 1;

            if byte == b'\n' {
                break;
            }
        }
        let mut end_offset: usize = self.data_offset - 1;

        if end_offset > 1 && self.data[end_offset - 1] == b'\r' {
            end_offset -= 1;
        }
        Some(&self.data[start_offset..end_offset])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x31, 0x0d, 0x0a, 0x6d, 0x61, 0x69, 0x6e, 0x0d, 0x0a, 0x63, 0x09, 0x6e, 0x09, 0x61,
            0x09, 0x65, 0x09, 0x74, 0x09, 0x61, 0x76, 0x09, 0x6f, 0x76, 0x09, 0x6d, 0x09, 0x75,
            0x09, 0x70, 0x0d, 0x0a, 0x63, 0x61, 0x73, 0x65, 0x09, 0x65, 0x76, 0x69, 0x64, 0x65,
            0x6e, 0x63, 0x65, 0x09, 0x64, 0x65, 0x73, 0x63, 0x72, 0x69, 0x70, 0x74, 0x69, 0x6f,
            0x6e, 0x09, 0x65, 0x78, 0x61, 0x6d, 0x69, 0x6e, 0x65, 0x72, 0x09, 0x6e, 0x6f, 0x74,
            0x65, 0x73, 0x09, 0x32, 0x30, 0x31, 0x34, 0x30, 0x38, 0x31, 0x37, 0x09, 0x4c, 0x69,
            0x6e, 0x75, 0x78, 0x09, 0x32, 0x30, 0x32, 0x35, 0x20, 0x39, 0x20, 0x31, 0x37, 0x20,
            0x31, 0x39, 0x20, 0x34, 0x36, 0x20, 0x31, 0x09, 0x32, 0x30, 0x32, 0x35, 0x20, 0x39,
            0x20, 0x31, 0x37, 0x20, 0x31, 0x39, 0x20, 0x34, 0x36, 0x20, 0x31, 0x09, 0x30, 0x0d,
            0x0a, 0x0d, 0x0a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
    }

    #[test]
    fn test_next() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_iter = EwfLinesIterator::new(&test_data);

        let expected_line: Vec<u8> = vec![0x31];
        assert_eq!(test_iter.next(), Some(expected_line.as_slice()));

        let expected_line: Vec<u8> = vec![0x6d, 0x61, 0x69, 0x6e];
        assert_eq!(test_iter.next(), Some(expected_line.as_slice()));
    }
}
