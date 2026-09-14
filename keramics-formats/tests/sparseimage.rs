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

use std::path::PathBuf;

use keramics_core::{DataStreamReference, ErrorTrace, open_os_data_stream};
use keramics_formats::cdsaencr::CdsaEncrCredential;
use keramics_formats::sparseimage::SparseImageFile;

mod util;

use util::read_data_stream;

fn open_file(path: &PathBuf) -> Result<SparseImageFile, ErrorTrace> {
    let data_stream: DataStreamReference = match open_os_data_stream(path) {
        Ok(data_stream) => data_stream,
        Err(mut error) => {
            keramics_core::error_trace_add_frame!(error, "Unable to open data stream");
            return Err(error);
        }
    };
    let mut file: SparseImageFile = SparseImageFile::new();

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
fn read_file() -> Result<(), ErrorTrace> {
    let path_buf: PathBuf = PathBuf::from("../test_data/sparseimage/hfsplus.sparseimage");
    let file: SparseImageFile = open_file(&path_buf)?;
    let data_stream: DataStreamReference = file.get_data_stream().unwrap();

    let (media_offset, md5_hash): (u64, String) = read_data_stream(&data_stream)?;

    assert_eq!(media_offset, file.get_media_size());
    assert_eq!(md5_hash.as_str(), "22c35335e6fafcbfc2ef21f1839f228d");

    Ok(())
}

#[test]
fn read_file_encrypted() -> Result<(), ErrorTrace> {
    let path_buf: PathBuf = PathBuf::from("../test_data/sparseimage/hfsplus_aes128.sparseimage");
    let mut file: SparseImageFile = open_file(&path_buf)?;

    // Using the key data to bypass key derivation.
    let credentials: Vec<CdsaEncrCredential> = vec![CdsaEncrCredential::KeyData {
        identifier: vec![
            0x21, 0xd5, 0x5b, 0x47, 0x4b, 0x3b, 0x41, 0x2f, 0xb4, 0xaf, 0xb9, 0xde, 0xb6, 0x46,
            0x54, 0x2c,
        ],
        data: vec![
            0xb7, 0x26, 0x12, 0x70, 0xb5, 0xb2, 0x0a, 0x0b, 0xa7, 0xd9, 0x2f, 0xd0, 0x39, 0xc6,
            0xe7, 0x71, 0xb7, 0x26, 0x12, 0x70, 0xb5, 0xb2, 0x0a, 0x0b, 0xa7, 0xd9, 0x2f, 0xd0,
            0x39, 0xc6, 0xe7, 0x71, 0x00, 0x00, 0x00, 0x00,
        ],
    }];
    file.unlock(&credentials)?;

    let data_stream: DataStreamReference = file.get_data_stream().unwrap();

    let (media_offset, md5_hash): (u64, String) = read_data_stream(&data_stream)?;

    assert_eq!(media_offset, file.get_media_size());
    assert_eq!(md5_hash.as_str(), "52da5f232d3910a366379bf4c3f004aa");

    Ok(())
}
