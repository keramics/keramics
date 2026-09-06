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
use keramics_formats::qcow::{QcowCredential, QcowFile};

mod util;

use util::read_data_stream;

fn open_file(path: &PathBuf) -> Result<QcowFile, ErrorTrace> {
    let data_stream: DataStreamReference = match open_os_data_stream(path) {
        Ok(data_stream) => data_stream,
        Err(error) => {
            return Err(keramics_core::error_trace_new_with_error!(
                "Unable to open data stream",
                error
            ));
        }
    };
    let mut file: QcowFile = QcowFile::new();

    match file.read_data_stream(&data_stream) {
        Ok(_) => {}
        Err(mut error) => {
            keramics_core::error_trace_add_frame!(
                error,
                "Unable to read QCOW file from data stream"
            );
            return Err(error);
        }
    }
    Ok(file)
}

#[test]
fn read_file_qcow() -> Result<(), ErrorTrace> {
    let path_buf: PathBuf = PathBuf::from("../test_data/qcow/ext2.qcow");
    let file: QcowFile = open_file(&path_buf)?;
    let data_stream: DataStreamReference = file.get_data_stream().unwrap();

    let (media_offset, md5_hash): (u64, String) = read_data_stream(&data_stream)?;

    assert_eq!(media_offset, file.get_media_size());
    assert_eq!(md5_hash.as_str(), "b1760d0b35a512ef56970df4e6f8c5d6");

    Ok(())
}

#[test]
fn read_file_qcow2() -> Result<(), ErrorTrace> {
    let path_buf: PathBuf = PathBuf::from("../test_data/qcow/ext2.qcow2");
    let file: QcowFile = open_file(&path_buf)?;
    let data_stream: DataStreamReference = file.get_data_stream().unwrap();

    let (media_offset, md5_hash): (u64, String) = read_data_stream(&data_stream)?;

    assert_eq!(media_offset, file.get_media_size());
    assert_eq!(md5_hash.as_str(), "b1760d0b35a512ef56970df4e6f8c5d6");

    Ok(())
}

#[test]
fn read_file_qcow2_aes128_encrypted() -> Result<(), ErrorTrace> {
    let path_buf: PathBuf = PathBuf::from("../test_data/qcow/ext2.qcow2");
    let mut file: QcowFile = open_file(&path_buf)?;
    let credentials: Vec<QcowCredential> = vec![QcowCredential::Passphrase(b"KeRaMiCs".to_vec())];
    file.unlock(&credentials)?;

    let data_stream: DataStreamReference = file.get_data_stream().unwrap();

    let (media_offset, md5_hash): (u64, String) = read_data_stream(&data_stream)?;

    assert_eq!(media_offset, file.get_media_size());
    assert_eq!(md5_hash.as_str(), "b1760d0b35a512ef56970df4e6f8c5d6");

    Ok(())
}

#[test]
fn read_file_qcow2_zlib_compressed() -> Result<(), ErrorTrace> {
    let path_buf: PathBuf = PathBuf::from("../test_data/qcow/ext2_zlib.qcow2");
    let file: QcowFile = open_file(&path_buf)?;
    let data_stream: DataStreamReference = file.get_data_stream().unwrap();

    let (media_offset, md5_hash): (u64, String) = read_data_stream(&data_stream)?;

    assert_eq!(media_offset, file.get_media_size());
    assert_eq!(md5_hash.as_str(), "b1760d0b35a512ef56970df4e6f8c5d6");

    Ok(())
}

// TODO: add test with backing file.
