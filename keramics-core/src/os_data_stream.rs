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

use std::fs::{File, Metadata};
use std::io;
use std::io::{Read, Seek, SeekFrom};
use std::sync::{Arc, RwLock};

use super::data_stream::{DataStream, DataStreamReference};

impl DataStream for File {
    /// Retrieves the size of the data.
    fn get_size(&mut self) -> io::Result<u64> {
        let metadata: Metadata = self.metadata()?;

        Ok(metadata.len())
    }

    /// Reads data at the current position.
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        Read::read(self, buf)
    }

    /// Sets the current position of the data.
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        Seek::seek(self, pos)
    }
}

/// Opens a new operating system data stream.
pub fn open_os_data_stream(path: &str) -> io::Result<DataStreamReference> {
    let file: File = File::open(path)?;

    Ok(Arc::new(RwLock::new(file)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_size() -> io::Result<()> {
        let mut file: File = File::open("../test_data/file.txt")?;

        let size: u64 = file.get_size()?;
        assert_eq!(size, 202);

        Ok(())
    }

    #[test]
    fn test_open_os_data_stream() -> io::Result<()> {
        let _ = open_os_data_stream("../test_data/file.txt")?;

        Ok(())
    }
}
