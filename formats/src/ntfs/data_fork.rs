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
use std::sync::{Arc, RwLock};

use core::{DataStreamReference, FakeDataStream};
use types::Ucs2String;

use super::block_stream::NtfsBlockStream;
use super::mft_attribute::NtfsMftAttribute;

/// New Technologies File System (NTFS) data fork.
pub struct NtfsDataFork<'a> {
    /// The data stream.
    data_stream: DataStreamReference,

    /// Cluster block size.
    cluster_block_size: u32,

    /// The $DATA attribute.
    data_attribute: &'a NtfsMftAttribute,
}

impl<'a> NtfsDataFork<'a> {
    /// Creates a new data fork.
    pub fn new(
        data_stream: &DataStreamReference,
        cluster_block_size: u32,
        data_attribute: &'a NtfsMftAttribute,
    ) -> Self {
        Self {
            data_stream: data_stream.clone(),
            cluster_block_size: cluster_block_size,
            data_attribute: data_attribute,
        }
    }

    /// Retrieves the data stream.
    pub fn get_data_stream(&self) -> io::Result<DataStreamReference> {
        if self.data_attribute.is_resident() {
            let data_stream: FakeDataStream = FakeDataStream::new(
                &self.data_attribute.resident_data,
                self.data_attribute.data_size,
            );
            Ok(Arc::new(RwLock::new(data_stream)))
        } else {
            let mut block_stream: NtfsBlockStream = NtfsBlockStream::new(self.cluster_block_size);
            block_stream.open(&self.data_stream, self.data_attribute)?;

            Ok(Arc::new(RwLock::new(block_stream)))
        }
    }

    /// Retrieves the name from the directory entry $DATA attribute.
    pub fn get_name(&self) -> Option<&Ucs2String> {
        self.data_attribute.name.as_ref()
    }
}

// TODO: add tests.
