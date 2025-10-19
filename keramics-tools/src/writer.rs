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
use std::io::Write;
use std::path::PathBuf;

use keramics_core::{DataStreamReference, ErrorTrace};

/// Writes a data stream to a file.
pub struct DataStreamWriter<'a> {
    /// Target (or destination) path.
    target: &'a PathBuf,

    /// Number of data streams written.
    pub number_of_streams_written: usize,
}

impl<'a> DataStreamWriter<'a> {
    const READ_BUFFER_SIZE: usize = 65536;

    /// Creates a new data stream writer.
    pub fn new(target: &'a PathBuf) -> Self {
        Self {
            target: target,
            number_of_streams_written: 0,
        }
    }

    /// Writes a data stream to a file.
    pub fn write_data_stream(
        &mut self,
        data_stream: &DataStreamReference,
    ) -> Result<(), ErrorTrace> {
        let mut output_path = self.target.clone();
        output_path.push("keramics.bin");

        let mut output_file: File = match File::create(output_path) {
            Ok(file) => file,
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to create output file",
                    error
                ));
            }
        };
        let mut data: [u8; DataStreamWriter::READ_BUFFER_SIZE] =
            [0; DataStreamWriter::READ_BUFFER_SIZE];

        match data_stream.write() {
            Ok(mut data_stream) => {
                loop {
                    let read_count = match data_stream.read(&mut data) {
                        Ok(read_count) => read_count,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to read data stream"
                            );
                            return Err(error);
                        }
                    };
                    if read_count == 0 {
                        break;
                    }
                    let write_count: usize = match output_file.write(&data[..read_count]) {
                        Ok(write_count) => write_count,
                        Err(error) => {
                            return Err(keramics_core::error_trace_new_with_error!(
                                "Unable to write to output file",
                                error
                            ));
                        }
                    };
                    if write_count != read_count {
                        return Err(keramics_core::error_trace_new!(
                            "Unable to write all data to output file"
                        ));
                    }
                }
                self.number_of_streams_written += 1;
            }
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to obtain write lock on data stream",
                    error
                ));
            }
        };
        Ok(())
    }
}

// TODO: add tests
