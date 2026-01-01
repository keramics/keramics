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

use std::cmp::min;
use std::io::SeekFrom;
use std::sync::{Arc, RwLock};

use keramics_core::{DataStream, DataStreamReference, ErrorTrace};
use keramics_types::Uuid;

use crate::fake_file_resolver::FakeFileResolver;
use crate::file_resolver::FileResolverReference;
use crate::lru_cache::LruCache;
use crate::path_component::PathComponent;

use super::block_range::{PdiBlockRange, PdiBlockRangeType};
use super::enums::PdiExtentType;
use super::extent_file::PdiExtentFile;
use super::image_extent::PdiImageExtent;
use super::sparse_file::PdiSparseFile;

/// Parallels Disk Image (PDI) layer.
pub struct PdiImageLayer {
    /// File resolver.
    file_resolver: FileResolverReference,

    /// Identifier.
    pub identifier: Uuid,

    /// Extents.
    extents: Vec<PdiImageExtent>,

    /// Extent file cache.
    extent_file_cache: LruCache<u64, PdiExtentFile>,

    /// Parent identifier.
    parent_identifier: Option<Uuid>,

    /// Parent layer.
    parent_layer: Option<Arc<RwLock<PdiImageLayer>>>,

    /// The current offset.
    current_offset: u64,

    /// Media size.
    pub media_size: u64,
}

impl PdiImageLayer {
    /// Creates a new image layer.
    pub(super) fn new(
        identifier: &Uuid,
        parent_identifier: Option<&Uuid>,
        media_size: u64,
    ) -> Self {
        Self {
            file_resolver: FileResolverReference::new(Box::new(FakeFileResolver::new())),
            identifier: identifier.clone(),
            parent_identifier: parent_identifier.cloned(),
            parent_layer: None,
            extents: Vec::new(),
            extent_file_cache: LruCache::new(16),
            current_offset: 0,
            media_size,
        }
    }

    /// Adds an extent.
    pub(super) fn add_extent(
        &mut self,
        offset: u64,
        size: u64,
        file_name: &str,
        extent_type: PdiExtentType,
    ) {
        let extent: PdiImageExtent = PdiImageExtent::new(offset, size, file_name, extent_type);
        self.extents.push(extent);
    }

    /// Retrieves a specific extent file.
    fn get_extent_file(&mut self, extent_index: usize) -> Result<&mut PdiExtentFile, ErrorTrace> {
        let lookup_extent_index: u64 = extent_index as u64;

        if !self.extent_file_cache.contains(&lookup_extent_index) {
            let extent: &PdiImageExtent = match self.extents.get(extent_index) {
                Some(extent) => extent,
                None => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unable to retrieve extent: {}",
                        extent_index
                    )));
                }
            };
            let path_components: [PathComponent; 1] =
                [PathComponent::from(extent.file_name.as_str())];

            let data_stream: DataStreamReference =
                match self.file_resolver.get_data_stream(&path_components) {
                    Ok(Some(data_stream)) => data_stream,
                    Ok(None) => {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Missing extent file: {}",
                            extent.file_name
                        )));
                    }
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!("Unable to open extent file: {}", extent.file_name)
                        );
                        return Err(error);
                    }
                };
            let extent_file: PdiExtentFile = match &extent.extent_type {
                PdiExtentType::Sparse => {
                    let mut sparse_file: PdiSparseFile = PdiSparseFile::new();

                    match sparse_file.read_data_stream(&data_stream) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!("Unable to open sparse extent file: {}", extent.file_name)
                            );
                            return Err(error);
                        }
                    }
                    PdiExtentFile::Sparse(sparse_file)
                }
                PdiExtentType::Raw => PdiExtentFile::Raw(data_stream),
            };
            self.extent_file_cache
                .insert(lookup_extent_index, extent_file);
        }
        match self.extent_file_cache.get_mut(&lookup_extent_index) {
            Some(extent_file) => Ok(extent_file),
            None => Err(keramics_core::error_trace_new!(format!(
                "Unable to retrieve extent: {} from cache",
                extent_index
            ))),
        }
    }

    /// Opens an image layer.
    pub fn open(&mut self, file_resolver: &FileResolverReference) -> Result<(), ErrorTrace> {
        self.file_resolver = file_resolver.clone();

        Ok(())
    }

    /// Reads media data based on the extents.
    fn read_data_from_extents(&mut self, data: &mut [u8]) -> Result<usize, ErrorTrace> {
        let read_size: usize = data.len();
        let mut data_offset: usize = 0;
        let mut media_offset: u64 = self.current_offset;

        let mut extent_index: usize = 0;
        let mut extent_offset: u64 = self.current_offset;

        // TODO: optimize extent lookup
        for extent in self.extents.iter() {
            if extent_offset < extent.size {
                break;
            }
            extent_index += 1;
            extent_offset -= extent.size;
        }
        if extent_index >= self.extents.len() {
            return Err(keramics_core::error_trace_new!(format!(
                "Invalid media offset: {} (0x{:08x}) value out of bounds",
                media_offset, media_offset
            )));
        }
        let mut extent_size: u64 = match self.extents.get(extent_index) {
            Some(extent) => extent.size,
            None => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unable to retrieve extent: {}",
                    extent_index
                )));
            }
        };
        while data_offset < read_size {
            let extent_file: &mut PdiExtentFile = match self.get_extent_file(extent_index) {
                Ok(extent_file) => extent_file,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to retrieve extent file: {}", extent_index)
                    );
                    return Err(error);
                }
            };
            let extent_remainder_size: u64 = extent_size - extent_offset;
            let extent_read_size: usize =
                min(read_size - data_offset, extent_remainder_size as usize);

            let range_read_count: usize = match extent_file {
                PdiExtentFile::Raw(data_stream) => {
                    let data_end_offset: usize = data_offset + extent_read_size;

                    keramics_core::data_stream_read_at_position!(
                        data_stream,
                        &mut data[data_offset..data_end_offset],
                        SeekFrom::Start(extent_offset)
                    )
                }
                PdiExtentFile::Sparse(sparse_file) => {
                    let mut result: Result<Option<&PdiBlockRange>, ErrorTrace> =
                        sparse_file.block_tree.get_value(extent_offset);

                    if result == Ok(None) {
                        match sparse_file.read_block_allocation_table_entry(extent_offset) {
                            Ok(_) => {}
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    "Unable to read block allocation table entry"
                                );
                                return Err(error);
                            }
                        }
                        result = sparse_file.block_tree.get_value(extent_offset);
                    }
                    let block_range: &PdiBlockRange = match result {
                        Ok(Some(block_range)) => block_range,
                        Ok(None) => {
                            return Err(keramics_core::error_trace_new!(format!(
                                "Missing block range for offset: {} (0x{:08x})",
                                extent_offset, extent_offset
                            )));
                        }
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!(
                                    "Unable to retrieve block range for offset: {} (0x{:08x})",
                                    extent_offset, extent_offset
                                )
                            );
                            return Err(error);
                        }
                    };
                    let range_relative_offset: u64 = extent_offset - block_range.extent_offset;
                    let range_remainder_size: u64 = block_range.size - range_relative_offset;
                    let range_read_size: usize =
                        min(extent_read_size, range_remainder_size as usize);
                    let data_end_offset: usize = data_offset + range_read_size;

                    match block_range.range_type {
                        PdiBlockRangeType::InFile => match sparse_file.data_stream.as_ref() {
                            Some(data_stream) => {
                                keramics_core::data_stream_read_at_position!(
                                    data_stream,
                                    &mut data[data_offset..data_end_offset],
                                    SeekFrom::Start(
                                        block_range.data_offset + range_relative_offset
                                    )
                                )
                            }
                            None => {
                                return Err(keramics_core::error_trace_new!(format!(
                                    "Missing extent file: {} data stream",
                                    extent_index
                                )));
                            }
                        },
                        PdiBlockRangeType::InParentOrSparse => match self.parent_layer.as_ref() {
                            Some(parent_layer) => {
                                keramics_core::data_stream_read_at_position!(
                                    parent_layer,
                                    &mut data[data_offset..data_end_offset],
                                    SeekFrom::Start(media_offset)
                                )
                            }
                            None => {
                                data[data_offset..data_end_offset].fill(0);

                                range_read_size
                            }
                        },
                    }
                }
            };
            data_offset += range_read_count;
            extent_offset += range_read_count as u64;
            media_offset += range_read_count as u64;

            if media_offset >= self.media_size {
                break;
            }
            if extent_offset >= extent_size {
                extent_index += 1;

                extent_offset = 0;
                extent_size = match self.extents.get(extent_index) {
                    Some(extent) => extent.size,
                    None => {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Unable to retrieve extent: {}",
                            extent_index
                        )));
                    }
                };
            }
        }
        Ok(data_offset)
    }

    /// Sets the parent layer.
    pub fn set_parent(
        &mut self,
        parent_layer: &Arc<RwLock<PdiImageLayer>>,
    ) -> Result<(), ErrorTrace> {
        let parent_identifier: &Uuid = match &self.parent_identifier {
            Some(parent_identifier) => parent_identifier,
            None => {
                return Err(keramics_core::error_trace_new!("Missing parent identifier"));
            }
        };
        match parent_layer.read() {
            Ok(image_layer) => {
                if *parent_identifier != image_layer.identifier {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Parent identifier: {} does not match identifier of parent layer: {}",
                        parent_identifier, image_layer.identifier,
                    )));
                }
            }
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to obtain read lock on parent layer",
                    error
                ));
            }
        }
        self.parent_layer = Some(parent_layer.clone());

        Ok(())
    }
}

impl DataStream for PdiImageLayer {
    /// Retrieves the current position.
    fn get_offset(&mut self) -> Result<u64, ErrorTrace> {
        Ok(self.current_offset)
    }

    /// Retrieves the size of the data.
    fn get_size(&mut self) -> Result<u64, ErrorTrace> {
        Ok(self.media_size)
    }

    /// Reads data at the current position.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ErrorTrace> {
        if self.current_offset >= self.media_size {
            return Ok(0);
        }
        let remaining_media_size: u64 = self.media_size - self.current_offset;
        let mut read_size: usize = buf.len();

        if (read_size as u64) > remaining_media_size {
            read_size = remaining_media_size as usize;
        }
        let read_count: usize = match self.read_data_from_extents(&mut buf[..read_size]) {
            Ok(read_count) => read_count,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read data from extents");
                return Err(error);
            }
        };
        self.current_offset += read_count as u64;

        Ok(read_count)
    }

    /// Sets the current position of the data.
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, ErrorTrace> {
        self.current_offset = match pos {
            SeekFrom::Current(relative_offset) => {
                match self.current_offset.checked_add_signed(relative_offset) {
                    Some(offset) => offset,
                    None => {
                        return Err(keramics_core::error_trace_new!(
                            "Invalid offset value out of bounds"
                        ));
                    }
                }
            }
            SeekFrom::End(relative_offset) => {
                match self.media_size.checked_add_signed(relative_offset) {
                    Some(offset) => offset,
                    None => {
                        return Err(keramics_core::error_trace_new!(
                            "Invalid offset value out of bounds"
                        ));
                    }
                }
            }
            SeekFrom::Start(offset) => offset,
        };
        Ok(self.current_offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use crate::os_file_resolver::open_os_file_resolver;
    use crate::tests::get_test_data_path;

    fn get_image_layer() -> Result<PdiImageLayer, ErrorTrace> {
        let identifier: Uuid = Uuid::from_string("{5fbaabe3-6958-40ff-92a7-860e329aab41}")?;

        let mut image_layer: PdiImageLayer = PdiImageLayer::new(&identifier, None, 33554432);
        image_layer.add_extent(
            0,
            33554432,
            "hfsplus.hdd.0.{5fbaabe3-6958-40ff-92a7-860e329aab41}.hds",
            PdiExtentType::Sparse,
        );
        let path_string: String = get_test_data_path("pdi/hfsplus.hdd");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let file_resolver: FileResolverReference = open_os_file_resolver(&path_buf)?;

        image_layer.open(&file_resolver)?;

        Ok(image_layer)
    }

    // TODO: add test for read_data_from_extents
    // TODO: add test for set_parent

    #[test]
    fn test_get_offset() -> Result<(), ErrorTrace> {
        let mut image_layer: PdiImageLayer = get_image_layer()?;

        image_layer.seek(SeekFrom::Start(1024))?;

        let offset: u64 = image_layer.get_offset()?;
        assert_eq!(offset, 1024);

        Ok(())
    }

    #[test]
    fn test_get_size() -> Result<(), ErrorTrace> {
        let mut image_layer: PdiImageLayer = get_image_layer()?;

        let size: u64 = image_layer.get_size()?;
        assert_eq!(size, 33554432);

        Ok(())
    }

    #[test]
    fn test_seek_from_start() -> Result<(), ErrorTrace> {
        let mut image_layer: PdiImageLayer = get_image_layer()?;

        let offset: u64 = image_layer.seek(SeekFrom::Start(1024))?;
        assert_eq!(offset, 1024);

        Ok(())
    }

    #[test]
    fn test_seek_from_end() -> Result<(), ErrorTrace> {
        let mut image_layer: PdiImageLayer = get_image_layer()?;

        let offset: u64 = image_layer.seek(SeekFrom::End(-512))?;
        assert_eq!(offset, image_layer.media_size - 512);

        Ok(())
    }

    #[test]
    fn test_seek_from_current() -> Result<(), ErrorTrace> {
        let mut image_layer: PdiImageLayer = get_image_layer()?;

        let offset = image_layer.seek(SeekFrom::Start(1024))?;
        assert_eq!(offset, 1024);

        let offset: u64 = image_layer.seek(SeekFrom::Current(-512))?;
        assert_eq!(offset, 512);

        Ok(())
    }

    #[test]
    fn test_seek_before_zero() -> Result<(), ErrorTrace> {
        let mut image_layer: PdiImageLayer = get_image_layer()?;

        let result: Result<u64, ErrorTrace> = image_layer.seek(SeekFrom::Current(-512));
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_seek_beyond_size() -> Result<(), ErrorTrace> {
        let mut image_layer: PdiImageLayer = get_image_layer()?;

        let offset: u64 = image_layer.seek(SeekFrom::End(512))?;
        assert_eq!(offset, image_layer.media_size + 512);

        Ok(())
    }

    #[test]
    fn test_seek_and_read() -> Result<(), ErrorTrace> {
        let mut image_layer: PdiImageLayer = get_image_layer()?;
        image_layer.seek(SeekFrom::Start(1024))?;

        let mut data: Vec<u8> = vec![0; 512];
        let read_size: usize = image_layer.read(&mut data)?;
        assert_eq!(read_size, 512);

        let expected_data: Vec<u8> = vec![
            0x00, 0x53, 0x46, 0x48, 0x00, 0x00, 0xaa, 0x11, 0xaa, 0x11, 0x00, 0x30, 0x65, 0x43,
            0xec, 0xac, 0xb4, 0x4f, 0x62, 0xa5, 0x40, 0x2f, 0xe9, 0x46, 0x83, 0xd5, 0xb6, 0x67,
            0x66, 0xf4, 0x4a, 0xd2, 0x28, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xd7, 0xff,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        assert_eq!(data, expected_data);

        Ok(())
    }

    #[test]
    fn test_seek_and_read_beyond_media_size() -> Result<(), ErrorTrace> {
        let mut image_layer: PdiImageLayer = get_image_layer()?;
        image_layer.seek(SeekFrom::End(512))?;

        let mut data: Vec<u8> = vec![0; 512];
        let read_size: usize = image_layer.read(&mut data)?;
        assert_eq!(read_size, 0);

        Ok(())
    }
}
