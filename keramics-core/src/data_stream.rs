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
use std::io::SeekFrom;
use std::sync::{Arc, RwLock};

pub type DataStreamReference = Arc<RwLock<dyn DataStream>>;

/// Data stream trait.
pub trait DataStream {
    /// Retrieves the size of the data.
    fn get_size(&mut self) -> io::Result<u64>;

    /// Reads data at the current position.
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize>;

    /// Reads data at a specific position.
    #[inline(always)]
    fn read_at_position(&mut self, buf: &mut [u8], position: SeekFrom) -> io::Result<usize> {
        self.seek(position)?;
        self.read(buf)
    }

    /// Reads an exact amount of data at the current position.
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        let read_size: usize = buf.len();
        let read_count: usize = self.read(buf)?;

        if read_count != read_size {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Unable to read the exact amount",
            ));
        }
        Ok(())
    }

    /// Reads an exact amount of data at a specific position.
    #[inline(always)]
    fn read_exact_at_position(&mut self, buf: &mut [u8], position: SeekFrom) -> io::Result<u64> {
        let offset: u64 = self.seek(position)?;
        let read_size: usize = buf.len();
        let read_count: usize = self.read(buf)?;

        if read_count != read_size {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Unable to read the exact amount",
            ));
        }
        Ok(offset)
    }

    /// Sets the current position of the data.
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64>;
}
