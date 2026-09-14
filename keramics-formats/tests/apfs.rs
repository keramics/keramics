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
use keramics_formats::Path;
use keramics_formats::apfs::{
    ApfsContainer, ApfsCredential, ApfsFileEntry, ApfsFileSystem, ApfsVolume,
};

mod util;

use util::read_data_stream;

fn read_path(file_system: &ApfsFileSystem, path_string: &str) -> Result<(u64, String), ErrorTrace> {
    let path: Path = Path::from(path_string);
    let result: Option<ApfsFileEntry> = match file_system.get_file_entry_by_path(&path) {
        Ok(result) => result,
        Err(mut error) => {
            keramics_core::error_trace_add_frame!(
                error,
                format!("Unable to retrieve file entry: {}", path_string)
            );
            return Err(error);
        }
    };
    let file_entry: ApfsFileEntry = match result {
        Some(file_entry) => file_entry,
        None => {
            return Err(keramics_core::error_trace_new!(format!(
                "Missing data stream for file entry: {}",
                path_string
            )));
        }
    };
    let data_stream: DataStreamReference = file_entry.get_data_stream()?.unwrap();

    read_data_stream(&data_stream)
}

fn open_file_system(path: &PathBuf) -> Result<ApfsFileSystem, ErrorTrace> {
    let data_stream: DataStreamReference = match open_os_data_stream(path) {
        Ok(data_stream) => data_stream,
        Err(mut error) => {
            keramics_core::error_trace_add_frame!(error, "Unable to open data stream");
            return Err(error);
        }
    };
    let mut container: ApfsContainer = ApfsContainer::new();

    match container.read_data_stream(&data_stream) {
        Ok(_) => {}
        Err(mut error) => {
            keramics_core::error_trace_add_frame!(
                error,
                "Unable to read APFS container from data stream"
            );
            return Err(error);
        }
    };
    let mut volume: ApfsVolume = match container.get_volume_by_index(0) {
        Ok(volume) => volume,
        Err(mut error) => {
            keramics_core::error_trace_add_frame!(
                error,
                "Unable to retrieve volume: 0 from container"
            );
            return Err(error);
        }
    };
    if volume.is_locked() {
        let credentials: Vec<ApfsCredential> = vec![ApfsCredential::KeyData {
            identifier: vec![
                0x89, 0x30, 0x13, 0xc3, 0x08, 0x75, 0x47, 0x3e, 0x91, 0xfe, 0xdb, 0xd8, 0xb9, 0x43,
                0x7b, 0x29,
            ],
            data: vec![
                0x1e, 0x49, 0x0d, 0xb5, 0x92, 0xad, 0x6a, 0x19, 0x4e, 0x6e, 0xe8, 0x58, 0x64, 0x84,
                0xe2, 0x3f, 0xf6, 0xbd, 0x08, 0x11, 0x34, 0x2c, 0x16, 0x32, 0xc1, 0x8a, 0xb4, 0x25,
                0x2f, 0x76, 0x08, 0x16,
            ],
        }];
        match volume.unlock(&credentials) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to unlock volume: 0");
                return Err(error);
            }
        }
    }
    let file_system: ApfsFileSystem = match volume.get_file_system() {
        Ok(file_system) => file_system,
        Err(mut error) => {
            keramics_core::error_trace_add_frame!(
                error,
                "Unable to retrieve file system from volume: 0"
            );
            return Err(error);
        }
    };
    Ok(file_system)
}

#[test]
fn read_apfs_empty_file() -> Result<(), ErrorTrace> {
    let path_buf: PathBuf = PathBuf::from("../test_data/apfs/apfs.raw");
    let file_system: ApfsFileSystem = open_file_system(&path_buf)?;

    let (offset, md5_hash): (u64, String) = read_path(&file_system, "/emptyfile")?;
    assert_eq!(offset, 0);
    assert_eq!(md5_hash.as_str(), "d41d8cd98f00b204e9800998ecf8427e");

    Ok(())
}

#[test]
fn read_apfs_file_regular() -> Result<(), ErrorTrace> {
    let path_buf: PathBuf = PathBuf::from("../test_data/apfs/apfs.raw");
    let file_system: ApfsFileSystem = open_file_system(&path_buf)?;

    let (offset, md5_hash): (u64, String) = read_path(&file_system, "/testdir1/TestFile2")?;
    assert_eq!(offset, 11358);
    assert_eq!(md5_hash.as_str(), "3b83ef96387f14655fc854ddc3c6bd57");

    Ok(())
}

#[test]
fn read_encrypted_apfs_empty_file() -> Result<(), ErrorTrace> {
    let path_buf: PathBuf = PathBuf::from("../test_data/apfs/apfs_encrypted.raw");
    let file_system: ApfsFileSystem = open_file_system(&path_buf)?;

    let (offset, md5_hash): (u64, String) = read_path(&file_system, "/emptyfile")?;
    assert_eq!(offset, 0);
    assert_eq!(md5_hash.as_str(), "d41d8cd98f00b204e9800998ecf8427e");

    Ok(())
}

#[test]
fn read_encrypted_apfs_file_regular() -> Result<(), ErrorTrace> {
    let path_buf: PathBuf = PathBuf::from("../test_data/apfs/apfs_encrypted.raw");
    let file_system: ApfsFileSystem = open_file_system(&path_buf)?;

    let (offset, md5_hash): (u64, String) = read_path(&file_system, "/testdir1/TestFile2")?;
    assert_eq!(offset, 11358);
    assert_eq!(md5_hash.as_str(), "3b83ef96387f14655fc854ddc3c6bd57");

    Ok(())
}
