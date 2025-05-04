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

use std::io;

use core::formatters::format_as_string;
use core::{open_os_data_stream, DataStreamReference};
use formats::ntfs::{NtfsFileEntry, NtfsFileSystem, NtfsPath};
use hashes::{DigestHashContext, Md5Context};

fn read_data_stream(data_stream: &DataStreamReference) -> io::Result<(u64, String)> {
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
        Err(error) => return Err(core::error_to_io_error!(error)),
    };
    let hash_value: Vec<u8> = md5_context.finalize();
    let hash_string: String = format_as_string(&hash_value);

    Ok((offset, hash_string))
}

fn read_path(file_system: &NtfsFileSystem, location: &str) -> io::Result<(u64, String)> {
    let path: NtfsPath = NtfsPath::from(location);
    let file_entry: NtfsFileEntry = file_system.get_file_entry_by_path(&path)?.unwrap();

    let data_stream: DataStreamReference = file_entry.get_data_stream()?.unwrap();

    read_data_stream(&data_stream)
}

fn open_file_system(location: &str) -> io::Result<NtfsFileSystem> {
    let mut file_system: NtfsFileSystem = NtfsFileSystem::new();

    let data_stream: DataStreamReference = open_os_data_stream(location)?;
    file_system.read_data_stream(&data_stream)?;

    Ok(file_system)
}

#[test]
fn read_ntfs_empty_file() -> io::Result<()> {
    let file_system: NtfsFileSystem = open_file_system("../test_data/ntfs/ntfs.raw")?;

    let (offset, md5_hash): (u64, String) = read_path(&file_system, "/emptyfile")?;
    assert_eq!(offset, 0);
    assert_eq!(md5_hash.as_str(), "d41d8cd98f00b204e9800998ecf8427e");

    Ok(())
}
