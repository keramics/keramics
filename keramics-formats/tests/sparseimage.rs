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
        Err(error) => {
            return Err(keramics_core::error_trace_new_with_error!(
                "Unable to open data stream",
                error
            ));
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
fn read_media() -> Result<(), ErrorTrace> {
    let path_buf: PathBuf = PathBuf::from("../test_data/sparseimage/hfsplus.sparseimage");
    let file: SparseImageFile = open_file(&path_buf)?;
    let data_stream: DataStreamReference = file.get_data_stream().unwrap();

    let (media_offset, md5_hash): (u64, String) = read_data_stream(&data_stream)?;

    assert_eq!(media_offset, file.get_media_size());
    assert_eq!(md5_hash.as_str(), "22c35335e6fafcbfc2ef21f1839f228d");

    Ok(())
}

#[test]
fn read_media_encrypted() -> Result<(), ErrorTrace> {
    let path_buf: PathBuf = PathBuf::from("../test_data/sparseimage/hfsplus_aes128.sparseimage");
    let mut file: SparseImageFile = open_file(&path_buf)?;
    let credentials: Vec<CdsaEncrCredential> =
        vec![CdsaEncrCredential::Passphrase(b"KeRaMiCs".to_vec())];
    file.unlock(&credentials)?;
    let data_stream: DataStreamReference = file.get_data_stream().unwrap();

    let (media_offset, md5_hash): (u64, String) = read_data_stream(&data_stream)?;

    assert_eq!(media_offset, file.get_media_size());
    assert_eq!(md5_hash.as_str(), "52da5f232d3910a366379bf4c3f004aa");

    Ok(())
}
