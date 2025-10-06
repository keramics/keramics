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

use keramics_core::formatters::format_as_string;
use keramics_core::{DataStream, DataStreamReference, ErrorTrace, open_os_data_stream};
use keramics_formats::sparseimage::SparseImageFile;
use keramics_hashes::{DigestHashContext, Md5Context};

use std::fs::File;
use std::io::Write;

fn read_media_from_file_with_output_file(
    file: &mut SparseImageFile,
) -> Result<(u64, String), ErrorTrace> {
    let mut data: Vec<u8> = vec![0; 512];
    let mut md5_context: Md5Context = Md5Context::new();
    let mut media_offset: u64 = 0;

    let mut output_file: File = match File::create("test.raw") {
        Ok(file) => file,
        Err(error) => {
            return Err(keramics_core::error_trace_new_with_error!(
                "Unable to create output file",
                error
            ));
        }
    };
    loop {
        let read_count = match file.read(&mut data) {
            Ok(read_count) => read_count,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to read from sparseimage file at offset {} (0x{:08x})",
                        media_offset, media_offset
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
        media_offset += read_count as u64;
    }
    let hash_value: Vec<u8> = md5_context.finalize();
    let hash_string: String = format_as_string(&hash_value);

    Ok((media_offset, hash_string))
}

fn read_media_from_file(file: &mut SparseImageFile) -> Result<(u64, String), ErrorTrace> {
    let mut data: Vec<u8> = vec![0; 35891];
    let mut md5_context: Md5Context = Md5Context::new();
    let mut media_offset: u64 = 0;

    loop {
        let read_count = match file.read(&mut data) {
            Ok(read_count) => read_count,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to read from sparseimage file at offset {} (0x{:08x})",
                        media_offset, media_offset
                    )
                );
                return Err(error);
            }
        };
        if read_count == 0 {
            break;
        }
        md5_context.update(&data[..read_count]);

        media_offset += read_count as u64;
    }
    let hash_value: Vec<u8> = md5_context.finalize();
    let hash_string: String = format_as_string(&hash_value);

    Ok((media_offset, hash_string))
}

fn open_file(path: &str) -> Result<SparseImageFile, ErrorTrace> {
    let mut file: SparseImageFile = SparseImageFile::new();

    let data_stream: DataStreamReference = open_os_data_stream(path)?;
    match file.read_data_stream(&data_stream) {
        Ok(_) => {}
        Err(mut error) => {
            keramics_core::error_trace_add_frame!(
                error,
                "Unable to read sparseimage file from data stream"
            );
            return Err(error);
        }
    }
    Ok(file)
}

#[test]
fn read_media() -> Result<(), ErrorTrace> {
    let mut file: SparseImageFile = open_file("../test_data/sparseimage/hfsplus.sparseimage")?;

    let (media_offset, md5_hash): (u64, String) = read_media_from_file(&mut file)?;
    assert_eq!(media_offset, file.media_size);
    assert_eq!(md5_hash.as_str(), "22c35335e6fafcbfc2ef21f1839f228d");

    Ok(())
}
