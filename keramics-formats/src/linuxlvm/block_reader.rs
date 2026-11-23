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

use std::cmp::{Ordering, min};
use std::io::SeekFrom;

use keramics_core::{DataStreamReference, ErrorTrace};

use crate::file_resolver::FileResolverReference;
use crate::path_component::PathComponent;
use crate::traits::BlockReader;

use super::extent::{LinuxLvmExtent, LinuxLvmExtentValues};

/// Linux Logical Volume Manager (LVM) block reader.
pub struct LinuxLvmBlockReader {
    /// File resolver.
    file_resolver: FileResolverReference,

    /// File names.
    file_names: Vec<PathComponent>,

    /// Extents.
    extents: Vec<LinuxLvmExtent>,

    /// The size.
    size: u64,
}

impl LinuxLvmBlockReader {
    /// Creates a new block reader.
    pub(super) fn new(
        file_resolver: &FileResolverReference,
        file_names: &[PathComponent],
        extents: &[LinuxLvmExtent],
        size: u64,
    ) -> Self {
        Self {
            file_resolver: file_resolver.clone(),
            file_names: file_names.to_vec(),
            extents: extents.to_vec(),
            size,
        }
    }
}

impl BlockReader for LinuxLvmBlockReader {
    /// Retrieves the size of the data.
    fn get_size(&self) -> u64 {
        self.size
    }

    /// Reads media data based on the extents.
    fn read_data_from_blocks(&mut self, data: &mut [u8], offset: u64) -> Result<usize, ErrorTrace> {
        let read_size: usize = data.len();
        let mut data_offset: usize = 0;
        let mut current_offset: u64 = offset;

        let mut extent_index: usize = match self.extents.binary_search_by(|extent| {
            let range_end_offset: u64 = extent.logical_offset + extent.size;

            if current_offset >= range_end_offset {
                Ordering::Less
            } else if current_offset < extent.logical_offset {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        }) {
            Ok(extent_index) => extent_index,
            Err(_) => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Missing extent for offset: {} (0x{:08x})",
                    current_offset, current_offset
                )));
            }
        };
        while data_offset < read_size {
            if current_offset >= self.size {
                break;
            }
            let extent: &LinuxLvmExtent = match self.extents.get(extent_index) {
                Some(extent) => extent,
                None => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unable to retrieve extent: {} for offset: {} (0x{:08x})",
                        extent_index, current_offset, current_offset,
                    )));
                }
            };
            let range_relative_offset: u64 = current_offset - extent.logical_offset;
            let range_remainder_size: u64 = extent.size - range_relative_offset;

            let range_read_size: usize =
                min(read_size - data_offset, range_remainder_size as usize);
            let data_end_offset: usize = data_offset + range_read_size;

            match &extent.values {
                LinuxLvmExtentValues::Stripe {
                    physical_offset,
                    physical_volume_index,
                } => {
                    // TODO: cache data streams
                    let file_name: &PathComponent =
                        match self.file_names.get(*physical_volume_index) {
                            Some(file_name) => file_name,
                            None => {
                                return Err(keramics_core::error_trace_new!(format!(
                                    "Missing physical volume: {} file name",
                                    physical_volume_index,
                                )));
                            }
                        };

                    let path_components: [PathComponent; 1] = [file_name.clone()];

                    let data_stream: DataStreamReference =
                        match self.file_resolver.get_data_stream(&path_components) {
                            Ok(Some(data_stream)) => data_stream,
                            Ok(None) => {
                                return Err(keramics_core::error_trace_new!(format!(
                                    "Missing data stream: {}",
                                    file_name
                                )));
                            }
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    format!("Unable to open file: {}", file_name)
                                );
                                return Err(error);
                            }
                        };
                    keramics_core::data_stream_read_exact_at_position!(
                        &data_stream,
                        &mut data[data_offset..data_end_offset],
                        SeekFrom::Start(physical_offset + range_relative_offset)
                    );
                }
            }
            data_offset = data_end_offset;
            current_offset += range_read_size as u64;
            extent_index += 1;
        }
        Ok(data_offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::HashMap;

    use keramics_core::open_fake_data_stream;

    use crate::file_resolver::FileResolver;

    /// In-memory file resolver for testing.
    struct TestFileResolver {
        /// Data streams indexed by path component.
        data_streams: HashMap<PathComponent, DataStreamReference>,
    }

    impl TestFileResolver {
        /// Creates a new test file resolver.
        fn new() -> Self {
            Self {
                data_streams: HashMap::new(),
            }
        }
    }

    impl FileResolver for TestFileResolver {
        /// Retrieves a data stream for the given path components.
        fn get_data_stream(
            &self,
            path_components: &[PathComponent],
        ) -> Result<Option<DataStreamReference>, ErrorTrace> {
            match path_components.first() {
                Some(path_component) => Ok(self.data_streams.get(path_component).cloned()),
                None => Ok(None),
            }
        }
    }

    fn build_test_file_resolver() -> FileResolverReference {
        let mut file_resolver: TestFileResolver = TestFileResolver::new();
        let pv0_name: PathComponent = PathComponent::from("pv0");
        let pv0_stream: DataStreamReference = open_fake_data_stream(b"0123456789ABCDEF");
        file_resolver.data_streams.insert(pv0_name, pv0_stream);

        let pv1_name: PathComponent = PathComponent::from("pv1");
        let pv1_stream: DataStreamReference = open_fake_data_stream(b"ghijklmnopqrstuv");
        file_resolver.data_streams.insert(pv1_name, pv1_stream);

        let boxed_file_resolver: Box<dyn FileResolver> = Box::new(file_resolver);
        FileResolverReference::new(boxed_file_resolver)
    }

    #[test]
    fn test_get_size() {
        let file_resolver: FileResolverReference = build_test_file_resolver();
        let block_reader: LinuxLvmBlockReader =
            LinuxLvmBlockReader::new(&file_resolver, &[], &[], 32);

        let size: u64 = block_reader.get_size();
        assert_eq!(size, 32);
    }

    #[test]
    fn test_read_data_from_blocks() -> Result<(), ErrorTrace> {
        let file_resolver: FileResolverReference = build_test_file_resolver();
        let file_names: [PathComponent; 2] =
            [PathComponent::from("pv0"), PathComponent::from("pv1")];
        let extents: [LinuxLvmExtent; 2] = [
            LinuxLvmExtent {
                logical_offset: 0,
                size: 8,
                values: LinuxLvmExtentValues::Stripe {
                    physical_offset: 4,
                    physical_volume_index: 0,
                },
            },
            LinuxLvmExtent {
                logical_offset: 8,
                size: 8,
                values: LinuxLvmExtentValues::Stripe {
                    physical_offset: 2,
                    physical_volume_index: 1,
                },
            },
        ];
        let mut block_reader: LinuxLvmBlockReader =
            LinuxLvmBlockReader::new(&file_resolver, &file_names, &extents, 16);

        // Test reading across two extents from different physical volumes.
        let mut data: [u8; 10] = [0; 10];
        let read_count: usize = block_reader.read_data_from_blocks(&mut data, 4)?;
        assert_eq!(read_count, 10);
        assert_eq!(&data, b"89ABijklmn");

        // Test reading past the end of the volume truncates at volume size.
        let mut tail_data: [u8; 8] = [0; 8];
        let read_count: usize = block_reader.read_data_from_blocks(&mut tail_data, 12)?;
        assert_eq!(read_count, 4);
        assert_eq!(&tail_data[0..4], b"mnop");

        Ok(())
    }

    #[test]
    fn test_read_data_from_blocks_with_errors() {
        let file_resolver: FileResolverReference = build_test_file_resolver();
        let file_names: [PathComponent; 2] = [
            PathComponent::from("pv0"),
            PathComponent::from("missing_pv"),
        ];
        let extents: [LinuxLvmExtent; 3] = [
            LinuxLvmExtent {
                logical_offset: 8,
                size: 4,
                values: LinuxLvmExtentValues::Stripe {
                    physical_offset: 0,
                    physical_volume_index: 0,
                },
            },
            LinuxLvmExtent {
                logical_offset: 12,
                size: 4,
                values: LinuxLvmExtentValues::Stripe {
                    physical_offset: 0,
                    physical_volume_index: 1,
                },
            },
            LinuxLvmExtent {
                logical_offset: 16,
                size: 4,
                values: LinuxLvmExtentValues::Stripe {
                    physical_offset: 0,
                    physical_volume_index: 5,
                },
            },
        ];
        let mut block_reader: LinuxLvmBlockReader =
            LinuxLvmBlockReader::new(&file_resolver, &file_names, &extents, 32);

        // Test with offset before first extent (missing extent).
        let mut data: [u8; 4] = [0; 4];
        let result: Result<usize, ErrorTrace> = block_reader.read_data_from_blocks(&mut data, 0);
        assert!(result.is_err());

        // Test with offset after last extent (missing extent).
        let result: Result<usize, ErrorTrace> = block_reader.read_data_from_blocks(&mut data, 24);
        assert!(result.is_err());

        // Test with missing data stream in file resolver.
        let result: Result<usize, ErrorTrace> = block_reader.read_data_from_blocks(&mut data, 12);
        assert!(result.is_err());

        // Test with out-of-bounds physical volume index.
        let result: Result<usize, ErrorTrace> = block_reader.read_data_from_blocks(&mut data, 16);
        assert!(result.is_err());

        // Test reading beyond available extents while still under total volume size.
        let mut wide_data: [u8; 16] = [0; 16];
        let extents_short: [LinuxLvmExtent; 1] = [LinuxLvmExtent {
            logical_offset: 0,
            size: 4,
            values: LinuxLvmExtentValues::Stripe {
                physical_offset: 0,
                physical_volume_index: 0,
            },
        }];
        let mut short_reader: LinuxLvmBlockReader =
            LinuxLvmBlockReader::new(&file_resolver, &file_names, &extents_short, 16);
        let result: Result<usize, ErrorTrace> =
            short_reader.read_data_from_blocks(&mut wide_data, 0);
        assert!(result.is_err());
    }
}
