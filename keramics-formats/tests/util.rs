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

use keramics_core::formatters::format_as_string;
use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_hashes::{DigestHashContext, Md5Context};

pub fn read_data_stream(data_stream: &DataStreamReference) -> Result<(u64, String), ErrorTrace> {
    let mut data: Vec<u8> = vec![0; 35891];
    let mut md5_context: Md5Context = Md5Context::new();
    let mut offset: u64 = 0;

    match data_stream.write() {
        Ok(mut data_stream) => loop {
            let read_count = data_stream.read(&mut data)?;
            if read_count == 0 {
                break;
            }
            md5_context.update(&data[..read_count]);

            offset += read_count as u64;
        },
        Err(error) => {
            return Err(keramics_core::error_trace_new_with_error!(
                "Unable to obtain write lock on data stream",
                error
            ));
        }
    }
    let hash_value: Vec<u8> = md5_context.finalize();
    let hash_string: String = format_as_string(&hash_value);

    Ok((offset, hash_string))
}

#[allow(dead_code)]
pub fn read_data_stream_with_output_file(
    data_stream: &DataStreamReference,
) -> Result<(u64, String), ErrorTrace> {
    let mut data: Vec<u8> = vec![0; 35891];
    let mut md5_context: Md5Context = Md5Context::new();
    let mut offset: u64 = 0;

    let mut output_file: File = match File::create("test.raw") {
        Ok(file) => file,
        Err(error) => {
            return Err(keramics_core::error_trace_new_with_error!(
                "Unable to create output file",
                error
            ));
        }
    };
    match data_stream.write() {
        Ok(mut data_stream) => loop {
            let read_count = match data_stream.read(&mut data) {
                Ok(read_count) => read_count,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to read from data stream at offset {} (0x{:08x})",
                            offset, offset
                        )
                    );
                    return Err(error);
                }
            };
            if read_count == 0 {
                break;
            }
            md5_context.update(&data[..read_count]);

            match output_file.write(&data[..read_count]) {
                Ok(write_count) => write_count,
                Err(error) => {
                    return Err(keramics_core::error_trace_new_with_error!(
                        "Unable to write to output file",
                        error
                    ));
                }
            };
            offset += read_count as u64;
        },
        Err(error) => {
            return Err(keramics_core::error_trace_new_with_error!(
                "Unable to obtain write lock on data stream",
                error
            ));
        }
    }
    let hash_value: Vec<u8> = md5_context.finalize();
    let hash_string: String = format_as_string(&hash_value);

    Ok((offset, hash_string))
}

#[allow(dead_code)]
pub fn read_data_stream_with_md5_per_block(
    data_stream: &DataStreamReference,
    block_size: usize,
) -> Result<(u64, String), ErrorTrace> {
    let mut data: Vec<u8> = vec![0; block_size];
    let mut md5_context: Md5Context = Md5Context::new();
    let mut offset: u64 = 0;

    match data_stream.write() {
        Ok(mut data_stream) => loop {
            let read_count = data_stream.read(&mut data)?;
            if read_count == 0 {
                break;
            }
            md5_context.update(&data[..read_count]);

            let next_offset: u64 = offset + (read_count as u64);

            let mut block_md5_context: Md5Context = Md5Context::new();
            block_md5_context.update(&data[..read_count]);
            let hash_value: Vec<u8> = block_md5_context.finalize();
            let hash_string: String = format_as_string(&hash_value);

            println!(
                "block 0x{:08x} - 0x{:08x}: MD5: {}",
                offset, next_offset, hash_string
            );
            offset = next_offset;
        },
        Err(error) => {
            return Err(keramics_core::error_trace_new_with_error!(
                "Unable to obtain write lock on data stream",
                error
            ));
        }
    }
    let hash_value: Vec<u8> = md5_context.finalize();
    let hash_string: String = format_as_string(&hash_value);

    Ok((offset, hash_string))
}
