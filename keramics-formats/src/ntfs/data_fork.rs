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

use std::io;

use keramics_core::DataStreamReference;
use keramics_types::Ucs2String;

use super::mft_attribute::NtfsMftAttribute;
use super::mft_attributes::NtfsMftAttributes;

/// New Technologies File System (NTFS) data fork.
pub struct NtfsDataFork<'a> {
    /// The data stream.
    data_stream: DataStreamReference,

    /// Cluster block size.
    cluster_block_size: u32,

    /// The MFT attributes.
    mft_attributes: &'a NtfsMftAttributes,

    /// The $DATA attribute.
    data_attribute: &'a NtfsMftAttribute,
}

impl<'a> NtfsDataFork<'a> {
    /// Creates a new data fork.
    pub fn new(
        data_stream: &DataStreamReference,
        cluster_block_size: u32,
        mft_attributes: &'a NtfsMftAttributes,
        data_attribute: &'a NtfsMftAttribute,
    ) -> Self {
        Self {
            data_stream: data_stream.clone(),
            cluster_block_size: cluster_block_size,
            mft_attributes: mft_attributes,
            data_attribute: data_attribute,
        }
    }

    /// Retrieves the data stream.
    pub fn get_data_stream(&self) -> io::Result<DataStreamReference> {
        match self.mft_attributes.get_data_stream_by_name(
            &self.data_attribute.name,
            &self.data_stream,
            self.cluster_block_size,
        )? {
            Some(data_stream) => Ok(data_stream),
            None => Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Missing data stream",
            )),
        }
    }

    /// Retrieves the name from the directory entry $DATA attribute.
    pub fn get_name(&self) -> Option<&Ucs2String> {
        self.data_attribute.name.as_ref()
    }
}

// TODO: add tests.
