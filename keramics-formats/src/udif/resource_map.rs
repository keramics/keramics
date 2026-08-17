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

use std::io::SeekFrom;

use keramics_core::{DataStreamReference, DebugTrace, ErrorTrace};
use keramics_types::{ByteString, bytes_to_u16_be};

use super::resource_descriptor::UdifResourceDescriptor;
use super::resource_map_entry::UdifResourceMapEntry;
use super::resource_map_header::UdifResourceMapHeader;
use super::resource_map_item::UdifResourceMapItem;
use super::resource_map_value::UdifResourceMapValue;

/// Universal Disk Image Format (UDIF) resource map.
pub struct UdifResourceMap {
    /// Entries.
    pub items: Vec<UdifResourceMapItem>,
}

impl UdifResourceMap {
    /// Creates a new resource map.
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Reads the resource map from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        keramics_core::debug_trace_structure!(UdifResourceMapHeader::debug_read_data(data));

        let mut resource_map_header: UdifResourceMapHeader = UdifResourceMapHeader::new();

        match resource_map_header.read_data(data) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read resource map header");
                return Err(error);
            }
        }
        let data_size: usize = data.len();

        let entries_list_offset: usize = resource_map_header.entries_list_offset as usize;

        if entries_list_offset < 28 || entries_list_offset > data_size {
            return Err(keramics_core::error_trace_new!(
                "Invalid entries list offset value out of bounds"
            ));
        }
        let names_list_offset: usize = resource_map_header.names_list_offset as usize;

        if names_list_offset < 28 || names_list_offset > data_size {
            return Err(keramics_core::error_trace_new!(
                "Invalid names list offset value out of bounds"
            ));
        }
        let mut data_offset: usize = entries_list_offset;

        let number_of_entries: usize = (bytes_to_u16_be!(data, data_offset) as usize) + 1;
        data_offset += 2;

        DebugTrace::static_scope(|debug_trace| {
            debug_trace.print_start("UdifResourceMap");
            debug_trace.print_field("number_of_entries", number_of_entries);
            debug_trace.print_end();
        });
        let data_end_offset: usize = data_offset + (number_of_entries * 8);

        if data_end_offset > data_size {
            return Err(keramics_core::error_trace_new!(
                "Invalid number of entries value out of bounds"
            ));
        }
        for entry_index in 0..number_of_entries {
            keramics_core::debug_trace_structure!(UdifResourceMapEntry::debug_read_data(
                &data[data_offset..]
            ));
            let mut resource_map_entry: UdifResourceMapEntry = UdifResourceMapEntry::new();

            match resource_map_entry.read_data(&data[data_offset..]) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to read resource map entry: {}", entry_index)
                    );
                    return Err(error);
                }
            }
            data_offset += 8;

            let mut resource_descriptors_offset: usize =
                (resource_map_entry.resource_descriptors_offset as usize) + entries_list_offset;

            if resource_descriptors_offset < data_end_offset
                || resource_descriptors_offset > data_size
            {
                return Err(keramics_core::error_trace_new!(format!(
                    "Invalid resource map entry: {} - resource descriptor offset value out of bounds",
                    entry_index
                )));
            }
            let mut resource_map_item: UdifResourceMapItem =
                UdifResourceMapItem::new(resource_map_entry.name);

            // TODO check upper bound of resource descriptors.
            for resource_descriptor_index in 0..resource_map_entry.number_of_resource_descriptors {
                keramics_core::debug_trace_structure!(UdifResourceDescriptor::debug_read_data(
                    &data[resource_descriptors_offset..]
                ));
                let mut resource_descriptor: UdifResourceDescriptor = UdifResourceDescriptor::new();

                match resource_descriptor.read_data(&data[resource_descriptors_offset..]) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!(
                                "Unable to read resource descriptor: {} of resource map entry: {}",
                                resource_descriptor_index, entry_index
                            )
                        );
                        return Err(error);
                    }
                }
                resource_descriptors_offset += 12;

                let mut resource_map_value: UdifResourceMapValue =
                    UdifResourceMapValue::new(resource_descriptor.data_offset);

                if resource_descriptor.name_offset != 0xffff {
                    let mut name_offset: usize =
                        names_list_offset + (resource_descriptor.name_offset as usize);

                    if name_offset > data_size - 1 {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Invalid resource map entry: {} - invalid resource descriptor: {} - name offset value out of bounds",
                            entry_index, resource_descriptor_index,
                        )));
                    }
                    let name_size: u8 = data[name_offset];
                    name_offset += 1;

                    let name_end_offset: usize = name_offset + (name_size as usize);

                    if name_end_offset > data_size {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Invalid resource map entry: {} - invalid resource descriptor: {} - name size value out of bounds",
                            entry_index, resource_descriptor_index,
                        )));
                    }
                    let mut name: ByteString = ByteString::new();
                    name.read_data(&data[name_offset..name_end_offset]);

                    resource_map_value.name = Some(name);
                }
                resource_map_item.values.push(resource_map_value);
            }
            self.items.push(resource_map_item);
        }
        Ok(())
    }

    /// Reads the resource map from a specific position in a data stream.
    pub fn read_at_position(
        &mut self,
        data_stream: &DataStreamReference,
        data_size: u32,
        position: SeekFrom,
    ) -> Result<(), ErrorTrace> {
        // Note that 65536 is an arbitrary chosen limit.
        if data_size < 28 || data_size > 65536 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported resource map data size: {} value out of bounds",
                data_size
            )));
        }
        let mut data: Vec<u8> = vec![0; data_size as usize];

        keramics_core::data_stream_read_exact_at_position_with_debug_trace_data!(
            "UdifResourceMap",
            data_stream,
            &mut data,
            data_size,
            position,
        );
        self.read_data(&data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_core::open_fake_data_stream;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x0a, 0x2c, 0x00, 0x00, 0x09, 0x2c, 0x00, 0x00,
            0x00, 0xd7, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0a, 0x00, 0x00, 0x00, 0x1c, 0x00, 0x6a,
            0x00, 0x01, 0x70, 0x6c, 0x73, 0x74, 0x00, 0x00, 0x00, 0x12, 0x62, 0x6c, 0x6b, 0x78,
            0x00, 0x03, 0x00, 0x1e, 0x00, 0x00, 0xff, 0xff, 0x50, 0x00, 0x00, 0x00, 0x00, 0x05,
            0x50, 0x90, 0xff, 0xff, 0x00, 0x00, 0x50, 0x00, 0x04, 0x0c, 0x00, 0x05, 0x50, 0x94,
            0x00, 0x00, 0x00, 0x20, 0x50, 0x00, 0x05, 0x2c, 0x00, 0x05, 0x50, 0x98, 0x00, 0x01,
            0x00, 0x40, 0x50, 0x00, 0x06, 0x4c, 0x00, 0x05, 0x50, 0x9c, 0x00, 0x02, 0x00, 0x5b,
            0x50, 0x00, 0x08, 0x0c, 0x00, 0x05, 0x50, 0xa0, 0x1f, 0x44, 0x72, 0x69, 0x76, 0x65,
            0x72, 0x20, 0x44, 0x65, 0x73, 0x63, 0x72, 0x69, 0x70, 0x74, 0x6f, 0x72, 0x20, 0x4d,
            0x61, 0x70, 0x20, 0x28, 0x44, 0x44, 0x4d, 0x20, 0x3a, 0x20, 0x30, 0x29, 0x1f, 0x41,
            0x70, 0x70, 0x6c, 0x65, 0x20, 0x28, 0x41, 0x70, 0x70, 0x6c, 0x65, 0x5f, 0x70, 0x61,
            0x72, 0x74, 0x69, 0x74, 0x69, 0x6f, 0x6e, 0x5f, 0x6d, 0x61, 0x70, 0x20, 0x3a, 0x20,
            0x31, 0x29, 0x1a, 0x64, 0x69, 0x73, 0x6b, 0x20, 0x69, 0x6d, 0x61, 0x67, 0x65, 0x20,
            0x28, 0x41, 0x70, 0x70, 0x6c, 0x65, 0x5f, 0x48, 0x46, 0x53, 0x20, 0x3a, 0x20, 0x32,
            0x29, 0x11, 0x20, 0x28, 0x41, 0x70, 0x70, 0x6c, 0x65, 0x5f, 0x46, 0x72, 0x65, 0x65,
            0x20, 0x3a, 0x20, 0x33, 0x29,
        ];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = UdifResourceMap::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.items.len(), 2);

        Ok(())
    }

    #[test]
    fn test_read_at_position() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();
        let test_data_size: u32 = test_data.len() as u32;
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut test_struct = UdifResourceMap::new();
        test_struct.read_at_position(&data_stream, test_data_size, SeekFrom::Start(0))?;

        assert_eq!(test_struct.items.len(), 2);

        Ok(())
    }
}
