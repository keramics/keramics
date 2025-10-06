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

use std::io::SeekFrom;
use std::sync::{Arc, RwLock};

use super::errors::ErrorTrace;

pub type DataStreamReference = Arc<RwLock<dyn DataStream>>;

/// Data stream trait.
pub trait DataStream {
    /// Retrieves the size of the data.
    fn get_size(&mut self) -> Result<u64, ErrorTrace>;

    /// Reads data at the current position.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ErrorTrace>;

    /// Reads data at a specific position.
    #[inline(always)]
    fn read_at_position(&mut self, buf: &mut [u8], pos: SeekFrom) -> Result<usize, ErrorTrace> {
        self.seek(pos)?;
        self.read(buf)
    }

    /// Reads an exact amount of data at the current position.
    #[inline(always)]
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), ErrorTrace> {
        let read_size: usize = buf.len();
        let read_count: usize = self.read(buf)?;

        if read_count != read_size {
            return Err(ErrorTrace::new(format!(
                "{}: Unable to read the exact amount",
                crate::error_trace_function!(),
            )));
        }
        Ok(())
    }

    /// Reads an exact amount of data at a specific position.
    #[inline(always)]
    fn read_exact_at_position(&mut self, buf: &mut [u8], pos: SeekFrom) -> Result<u64, ErrorTrace> {
        let offset: u64 = self.seek(pos)?;
        let read_size: usize = buf.len();
        let read_count: usize = self.read(buf)?;

        if read_count != read_size {
            return Err(ErrorTrace::new(format!(
                "{}: Unable to read the exact amount",
                crate::error_trace_function!(),
            )));
        }
        Ok(offset)
    }

    /// Sets the current position of the data.
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, ErrorTrace>;
}
