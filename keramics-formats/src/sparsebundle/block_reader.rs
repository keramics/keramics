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

use keramics_core::{DataStreamReference, ErrorTrace};

use crate::cdsaencr::CdsaEncrEncryptionContext;
use crate::file_resolver::FileResolverReference;
use crate::lru_cache::LruCache;
use crate::path_component::PathComponent;
use crate::traits::BlockReader;

/// Mac OS sparse bundle (.sparsebundle) block reader.
pub struct SparseBundleBlockReader {
    /// File resolver.
    file_resolver: FileResolverReference,

    /// Band size.
    band_size: u32,

    /// Band file cache.
    band_file_cache: LruCache<u64, DataStreamReference>,

    /// Encryption context.
    encryption_context: Option<CdsaEncrEncryptionContext>,

    /// Encrypted block size.
    encrypted_block_size: usize,

    /// Size.
    size: u64,
}

impl SparseBundleBlockReader {
    /// Creates a new storage media image.
    pub fn new(
        file_resolver: &FileResolverReference,
        band_size: u32,
        encryption_context: Option<&CdsaEncrEncryptionContext>,
        encrypted_block_size: usize,
        size: u64,
    ) -> Self {
        Self {
            file_resolver: file_resolver.clone(),
            band_size,
            band_file_cache: LruCache::new(16),
            encryption_context: encryption_context.cloned(),
            encrypted_block_size,
            size,
        }
    }
}

impl BlockReader for SparseBundleBlockReader {
    /// Retrieves the size of the data.
    fn get_size(&self) -> u64 {
        self.size
    }

    /// Reads data based on the block ranges.
    fn read_data_from_blocks(&mut self, data: &mut [u8], offset: u64) -> Result<usize, ErrorTrace> {
        let read_size: usize = data.len();
        let mut data_offset: usize = 0;
        let mut current_offset: u64 = offset;

        let mut band_number: u64 = current_offset / (self.band_size as u64);
        let band_offset: u64 = band_number * (self.band_size as u64);
        let mut range_relative_offset: u64 = current_offset - band_offset;
        let mut range_remainder_size: u64 = (self.band_size as u64) - range_relative_offset;

        while data_offset < read_size {
            if current_offset >= self.size {
                break;
            }
            if !self.band_file_cache.contains(&band_number) {
                let band_file_name: String = format!("{:x}", band_number);

                let path_components: [PathComponent; 2] = [
                    PathComponent::from("bands"),
                    PathComponent::from(&band_file_name),
                ];
                let data_stream: DataStreamReference =
                    match self.file_resolver.get_data_stream(&path_components) {
                        Ok(Some(data_stream)) => data_stream,
                        Ok(None) => {
                            return Err(keramics_core::error_trace_new!(format!(
                                "Missing band file: {}",
                                band_file_name
                            )));
                        }
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!("Unable to open band file: {}", band_file_name)
                            );
                            return Err(error);
                        }
                    };
                self.band_file_cache.insert(band_number, data_stream);
            }
            let data_stream: &DataStreamReference = match self.band_file_cache.get(&band_number) {
                Some(file) => file,
                None => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unable to retrieve band file: bands/{:x} from cache",
                        band_number
                    )));
                }
            };
            let range_read_size: usize =
                min(read_size - data_offset, range_remainder_size as usize);

            match &mut self.encryption_context {
                Some(encryption_context) => {
                    let range_data_end_offset: usize = data_offset + range_read_size;

                    let mut block_number: u64 = current_offset / (self.encrypted_block_size as u64);
                    let block_logical_offset: u64 =
                        block_number * (self.encrypted_block_size as u64);
                    let mut block_data_offset: usize =
                        (current_offset - block_logical_offset) as usize;

                    let mut block_physical_offset: u64 =
                        range_relative_offset - (block_data_offset as u64);
                    let mut block_remainder_size: usize =
                        self.encrypted_block_size - block_data_offset;

                    while data_offset < range_data_end_offset {
                        // TODO: cache decrypted block.
                        let mut encrypted_data: Vec<u8> = vec![0; self.encrypted_block_size];

                        keramics_core::data_stream_read_exact_at_position!(
                            data_stream,
                            &mut encrypted_data,
                            SeekFrom::Start(block_physical_offset)
                        );
                        let mut block_data: Vec<u8> = vec![0; self.encrypted_block_size];

                        match encryption_context.decrypt_block(
                            block_number as u32,
                            &encrypted_data,
                            &mut block_data,
                        ) {
                            Ok(_) => {}
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    format!("Unable to decrypt block: {}", block_number)
                                );
                                return Err(error);
                            }
                        }
                        let block_read_size: usize =
                            min(read_size - data_offset, block_remainder_size);

                        let data_end_offset: usize = data_offset + block_read_size;
                        let block_data_end_offset: usize = block_data_offset + block_read_size;

                        data[data_offset..data_end_offset]
                            .copy_from_slice(&block_data[block_data_offset..block_data_end_offset]);
                        data_offset = data_end_offset;
                        current_offset += block_read_size as u64;
                        block_number += 1;
                        block_physical_offset += self.encrypted_block_size as u64;
                        block_data_offset = 0;
                        block_remainder_size = self.encrypted_block_size;
                    }
                }
                None => {
                    let data_end_offset: usize = data_offset + range_read_size;

                    let read_count: usize = keramics_core::data_stream_read_at_position!(
                        data_stream,
                        &mut data[data_offset..data_end_offset],
                        SeekFrom::Start(range_relative_offset)
                    );
                    if read_count == 0 {
                        break;
                    }
                    data_offset += read_count;
                    current_offset += read_count as u64;
                }
            }
            band_number += 1;
            range_relative_offset = 0;
            range_remainder_size = self.band_size as u64;
        }
        Ok(data_offset)
    }
}
