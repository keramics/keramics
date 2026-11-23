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

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use keramics_core::{DataStreamReference, ErrorTrace};

/// Writes a data stream to a file.
pub struct DataStreamWriter {
    /// Number of data streams written.
    pub number_of_streams_written: usize,
}

impl DataStreamWriter {
    const READ_BUFFER_SIZE: usize = 65536;

    /// Creates a new data stream writer.
    pub fn new() -> Self {
        Self {
            number_of_streams_written: 0,
        }
    }

    /// Writes a data stream to a file.
    pub fn write_data_stream(
        &mut self,
        data_stream: &DataStreamReference,
        output_path: &PathBuf,
    ) -> Result<(), ErrorTrace> {
        let mut output_file: File = match File::create(output_path) {
            Ok(file) => file,
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to create output file",
                    error
                ));
            }
        };
        let mut data: [u8; Self::READ_BUFFER_SIZE] = [0; Self::READ_BUFFER_SIZE];

        match data_stream.write() {
            Ok(mut data_stream) => {
                loop {
                    let read_count: usize = match data_stream.read(&mut data) {
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

#[cfg(test)]
mod tests {
    use super::*;

    use std::env;
    use std::fs;
    use std::io::SeekFrom;
    use std::process;
    use std::sync::{Arc, RwLock};

    use keramics_core::{DataStream, open_fake_data_stream};

    /// Data stream that always fails to read.
    struct FailingDataStream {}

    impl DataStream for FailingDataStream {
        /// Retrieves the current position.
        fn get_offset(&mut self) -> Result<u64, ErrorTrace> {
            Ok(0)
        }

        /// Retrieves the size of the data.
        fn get_size(&mut self) -> Result<u64, ErrorTrace> {
            Ok(16)
        }

        /// Reads data at the current position.
        fn read(&mut self, _buf: &mut [u8]) -> Result<usize, ErrorTrace> {
            Err(keramics_core::error_trace_new!("Forced read failure"))
        }

        /// Sets the current position of the data.
        fn seek(&mut self, _pos: SeekFrom) -> Result<u64, ErrorTrace> {
            Ok(0)
        }
    }

    #[test]
    fn test_write_data_stream() -> Result<(), ErrorTrace> {
        let mut temporary_path: PathBuf = env::temp_dir();
        temporary_path.push(format!("keramics_writer_test_{}.bin", process::id()));

        let test_data: Vec<u8> = (0..70000).map(|index| (index % 251) as u8).collect();
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut data_stream_writer: DataStreamWriter = DataStreamWriter::new();
        assert_eq!(data_stream_writer.number_of_streams_written, 0);

        data_stream_writer.write_data_stream(&data_stream, &temporary_path)?;
        assert_eq!(data_stream_writer.number_of_streams_written, 1);

        let written_data: Vec<u8> = fs::read(&temporary_path).unwrap();
        assert_eq!(written_data, test_data);

        // Test with an empty data stream.
        let empty_stream: DataStreamReference = open_fake_data_stream(&[]);
        data_stream_writer.write_data_stream(&empty_stream, &temporary_path)?;
        assert_eq!(data_stream_writer.number_of_streams_written, 2);
        assert!(fs::read(&temporary_path).unwrap().is_empty());

        _ = fs::remove_file(&temporary_path);

        Ok(())
    }

    #[test]
    fn test_write_data_stream_errors() {
        let mut data_stream_writer: DataStreamWriter = DataStreamWriter::new();
        let data_stream: DataStreamReference = open_fake_data_stream(b"hello");

        let invalid_path: PathBuf = PathBuf::from("/nonexistent_keramics_dir_12345/out.bin");
        let result: Result<(), ErrorTrace> =
            data_stream_writer.write_data_stream(&data_stream, &invalid_path);
        assert!(result.is_err());
        assert_eq!(data_stream_writer.number_of_streams_written, 0);

        let mut temporary_path: PathBuf = env::temp_dir();
        temporary_path.push(format!("keramics_writer_fail_test_{}.bin", process::id()));

        let mut failing_data_stream: FailingDataStream = FailingDataStream {};
        assert_eq!(failing_data_stream.get_offset().unwrap(), 0);
        assert_eq!(failing_data_stream.get_size().unwrap(), 16);
        assert_eq!(failing_data_stream.seek(SeekFrom::Start(0)).unwrap(), 0);

        let failing_stream: DataStreamReference = Arc::new(RwLock::new(failing_data_stream));
        let result: Result<(), ErrorTrace> =
            data_stream_writer.write_data_stream(&failing_stream, &temporary_path);
        assert!(result.is_err());
        assert_eq!(data_stream_writer.number_of_streams_written, 0);

        _ = fs::remove_file(&temporary_path);
    }
}
