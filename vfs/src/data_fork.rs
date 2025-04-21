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

use core::DataStreamReference;
use formats::ntfs::NtfsDataFork;

use super::string::VfsString;

/// Virtual File System (VFS) data fork.
pub enum VfsDataFork<'a> {
    Ext(DataStreamReference),
    Ntfs(NtfsDataFork<'a>),
}

impl<'a> VfsDataFork<'a> {
    /// Retrieves the data stream.
    pub fn get_data_stream(&self) -> io::Result<DataStreamReference> {
        match self {
            VfsDataFork::Ext(data_stream) => Ok(data_stream.clone()),
            VfsDataFork::Ntfs(data_fork) => data_fork.get_data_stream(),
        }
    }

    /// Retrieves the name.
    pub fn get_name(&self) -> Option<VfsString> {
        match self {
            VfsDataFork::Ext(_) => None,
            VfsDataFork::Ntfs(data_fork) => match data_fork.get_name() {
                Some(name) => Some(VfsString::Ucs2(name.clone())),
                None => None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: add tests
}
