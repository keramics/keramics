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

use keramics_core::DataStreamReference;

use super::string::HfsString;

/// Hierarchical File System (HFS) extended attribute.
pub struct HfsExtendedAttribute {
    /// The name.
    name: HfsString,

    /// The data stream.
    data_stream: DataStreamReference,
}

impl HfsExtendedAttribute {
    /// Creates a new extended attribute.
    pub(super) fn new(name: &HfsString, data_stream: DataStreamReference) -> Self {
        Self {
            name: name.clone(),
            data_stream,
        }
    }

    /// Retrieves the default data stream.
    pub fn get_data_stream(&self) -> &DataStreamReference {
        &self.data_stream
    }

    /// Retrieves the name.
    pub fn get_name(&self) -> &HfsString {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_core::open_fake_data_stream;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x75, 0x6e, 0x63, 0x6f, 0x6e, 0x66, 0x69, 0x6e, 0x65, 0x64, 0x5f, 0x75, 0x3a, 0x6f,
            0x62, 0x6a, 0x65, 0x63, 0x74, 0x5f, 0x72, 0x3a, 0x75, 0x6e, 0x6c, 0x61, 0x62, 0x65,
            0x6c, 0x65, 0x64, 0x5f, 0x74, 0x3a, 0x73, 0x30, 0x00,
        ];
    }

    #[test]
    fn test_get_data_stream() {
        let test_name: HfsString = HfsString::from("test");
        let test_data: Vec<u8> = get_test_data();
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);
        let test_struct: HfsExtendedAttribute = HfsExtendedAttribute::new(&test_name, data_stream);

        let _ = test_struct.get_data_stream();
    }

    #[test]
    fn test_get_name() {
        let test_name: HfsString = HfsString::from("test");
        let test_data: Vec<u8> = get_test_data();
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);
        let test_struct: HfsExtendedAttribute = HfsExtendedAttribute::new(&test_name, data_stream);

        let name: &HfsString = test_struct.get_name();
        assert_eq!(name, &HfsString::from("test"));
    }
}
