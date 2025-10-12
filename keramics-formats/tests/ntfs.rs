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

use std::path::PathBuf;

use keramics_core::formatters::format_as_string;
use keramics_core::{DataStreamReference, ErrorTrace, open_os_data_stream};
use keramics_formats::ntfs::{NtfsFileEntry, NtfsFileSystem, NtfsPath};
use keramics_hashes::{DigestHashContext, Md5Context};

fn read_data_stream(data_stream: &DataStreamReference) -> Result<(u64, String), ErrorTrace> {
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
    };
    let hash_value: Vec<u8> = md5_context.finalize();
    let hash_string: String = format_as_string(&hash_value);

    Ok((offset, hash_string))
}

fn read_path(file_system: &NtfsFileSystem, path: &str) -> Result<(u64, String), ErrorTrace> {
    let path: NtfsPath = NtfsPath::from(path);
    let result: Option<NtfsFileEntry> = match file_system.get_file_entry_by_path(&path) {
        Ok(result) => result,
        Err(mut error) => {
            keramics_core::error_trace_add_frame!(
                error,
                format!("Unable to retrieve file entry: {}", path.to_string())
            );
            return Err(error);
        }
    };
    let file_entry: NtfsFileEntry = match result {
        Some(file_entry) => file_entry,
        None => {
            return Err(keramics_core::error_trace_new!(format!(
                "Missing file entry: {}",
                path.to_string()
            )));
        }
    };
    let data_stream: DataStreamReference = file_entry.get_data_stream()?.unwrap();

    read_data_stream(&data_stream)
}

fn open_file_system(path: &PathBuf) -> Result<NtfsFileSystem, ErrorTrace> {
    let data_stream: DataStreamReference = match open_os_data_stream(path) {
        Ok(data_stream) => data_stream,
        Err(error) => {
            return Err(keramics_core::error_trace_new_with_error!(
                "Unable to open data stream",
                error
            ));
        }
    };
    let mut file_system: NtfsFileSystem = NtfsFileSystem::new();

    match file_system.read_data_stream(&data_stream) {
        Ok(_) => {}
        Err(mut error) => {
            keramics_core::error_trace_add_frame!(
                error,
                "Unable to read NTFS file system from data stream"
            );
            return Err(error);
        }
    };
    Ok(file_system)
}

#[test]
fn read_ntfs_empty_file() -> Result<(), ErrorTrace> {
    let path_buf: PathBuf = PathBuf::from("../test_data/ntfs/ntfs.raw");
    let file_system: NtfsFileSystem = open_file_system(&path_buf)?;

    let (offset, md5_hash): (u64, String) = read_path(&file_system, "\\emptyfile")?;
    assert_eq!(offset, 0);
    assert_eq!(md5_hash.as_str(), "d41d8cd98f00b204e9800998ecf8427e");

    Ok(())
}
