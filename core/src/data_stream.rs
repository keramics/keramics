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

use std::fs::File;
use std::io;
use std::io::{Read, Seek, SeekFrom};

use super::shared_value::SharedValue;

pub type DataStreamReference = SharedValue<Box<dyn DataStream>>;

/// Data stream trait.
pub trait DataStream: Read + Seek {
    /// Retrieves the size of the data stream.
    fn get_size(&mut self) -> io::Result<u64>;

    /// Reads data at a specific position.
    #[inline(always)]
    fn read_at_position(&mut self, data: &mut [u8], position: SeekFrom) -> io::Result<usize> {
        self.seek(position)?;
        self.read(data)
    }

    /// Reads an exact amount of data at a specific position.
    #[inline(always)]
    fn read_exact_at_position(&mut self, data: &mut [u8], position: SeekFrom) -> io::Result<u64> {
        let offset: u64 = self.seek(position)?;
        self.read_exact(data)?;
        Ok(offset)
    }
}
