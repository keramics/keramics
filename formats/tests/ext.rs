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
use formats::ext::{ExtFileEntry, ExtFileSystem, ExtPath};
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

fn read_path(file_system: &ExtFileSystem, location: &str) -> io::Result<(u64, String)> {
    let path: ExtPath = ExtPath::from(location);
    let file_entry: ExtFileEntry = file_system.get_file_entry_by_path(&path)?.unwrap();

    let data_stream: DataStreamReference = file_entry.get_data_stream()?.unwrap();

    read_data_stream(&data_stream)
}

fn open_file_system(location: &str) -> io::Result<ExtFileSystem> {
    let mut file_system: ExtFileSystem = ExtFileSystem::new();

    let data_stream: DataStreamReference = open_os_data_stream(location)?;
    file_system.read_data_stream(&data_stream)?;

    Ok(file_system)
}

#[test]
fn read_ext2_file_empty() -> io::Result<()> {
    let file_system: ExtFileSystem = open_file_system("../test_data/ext/ext2.raw")?;

    let (offset, md5_hash): (u64, String) = read_path(&file_system, "/emptyfile")?;
    assert_eq!(offset, 0);
    assert_eq!(md5_hash.as_str(), "d41d8cd98f00b204e9800998ecf8427e");

    Ok(())
}

#[test]
fn read_ext2_file_regular() -> io::Result<()> {
    let file_system: ExtFileSystem = open_file_system("../test_data/ext/ext2.raw")?;

    let (offset, md5_hash): (u64, String) = read_path(&file_system, "/testdir1/TestFile2")?;
    assert_eq!(offset, 11358);
    assert_eq!(md5_hash.as_str(), "3b83ef96387f14655fc854ddc3c6bd57");

    Ok(())
}

#[test]
fn read_ext2_file_with_initial_sparse_extent() -> io::Result<()> {
    let file_system: ExtFileSystem = open_file_system("../test_data/ext/ext2.raw")?;

    let (offset, md5_hash): (u64, String) = read_path(&file_system, "/testdir1/initial_sparse1")?;
    assert_eq!(offset, 1048611);
    assert_eq!(md5_hash.as_str(), "c53dd591cf199ec5d692de2cbdb8559b");

    Ok(())
}

#[test]
fn read_ext2_file_with_trailing_sparse_extent() -> io::Result<()> {
    let file_system: ExtFileSystem = open_file_system("../test_data/ext/ext2.raw")?;

    let (offset, md5_hash): (u64, String) = read_path(&file_system, "/testdir1/trailing_sparse1")?;
    assert_eq!(offset, 1048576);
    assert_eq!(md5_hash.as_str(), "e0b16e3a6c58c67928b5895797fccaa0");

    Ok(())
}

#[test]
fn read_ext4_file_empty() -> io::Result<()> {
    let file_system: ExtFileSystem = open_file_system("../test_data/ext/ext4.raw")?;

    let (offset, md5_hash): (u64, String) = read_path(&file_system, "/emptyfile")?;
    assert_eq!(offset, 0);
    assert_eq!(md5_hash.as_str(), "d41d8cd98f00b204e9800998ecf8427e");

    Ok(())
}

#[test]
fn read_ext4_file_regular() -> io::Result<()> {
    let file_system: ExtFileSystem = open_file_system("../test_data/ext/ext4.raw")?;

    let (offset, md5_hash): (u64, String) = read_path(&file_system, "/testdir1/TestFile2")?;
    assert_eq!(offset, 11358);
    assert_eq!(md5_hash.as_str(), "3b83ef96387f14655fc854ddc3c6bd57");

    Ok(())
}

#[test]
fn read_ext4_file_with_inline_data() -> io::Result<()> {
    let file_system: ExtFileSystem = open_file_system("../test_data/ext/ext4.raw")?;

    let (offset, md5_hash): (u64, String) = read_path(&file_system, "/testdir1/testfile1")?;
    assert_eq!(offset, 9);
    assert_eq!(md5_hash.as_str(), "7fd0fc35a8c963bf34ba9d57427b3907");

    Ok(())
}

#[test]
fn read_ext4_file_with_initial_sparse_extent() -> io::Result<()> {
    let file_system: ExtFileSystem = open_file_system("../test_data/ext/ext4.raw")?;

    let (offset, md5_hash): (u64, String) = read_path(&file_system, "/testdir1/initial_sparse1")?;
    assert_eq!(offset, 1048611);
    assert_eq!(md5_hash.as_str(), "c53dd591cf199ec5d692de2cbdb8559b");

    Ok(())
}

#[test]
fn read_ext4_file_with_trailing_sparse_extent() -> io::Result<()> {
    let file_system: ExtFileSystem = open_file_system("../test_data/ext/ext4.raw")?;

    let (offset, md5_hash): (u64, String) = read_path(&file_system, "/testdir1/trailing_sparse1")?;
    assert_eq!(offset, 1048576);
    assert_eq!(md5_hash.as_str(), "e0b16e3a6c58c67928b5895797fccaa0");

    Ok(())
}

#[test]
fn read_ext4_file_with_uninitialized_extent() -> io::Result<()> {
    let file_system: ExtFileSystem = open_file_system("../test_data/ext/ext4.raw")?;

    let (offset, md5_hash): (u64, String) = read_path(&file_system, "/testdir1/uninitialized1")?;
    assert_eq!(offset, 4130);
    assert_eq!(md5_hash.as_str(), "5f43bd7169cfd72a1e0b5270970911f1");

    Ok(())
}
