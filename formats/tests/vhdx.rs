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
use std::io::Read;
use std::sync::{Arc, RwLock};

use core::formatters::format_as_string;
use core::{open_os_data_stream, DataStreamReference};
use formats::vhdx::VhdxFile;
use hashes::{DigestHashContext, Md5Context};

fn read_media_from_file(file: &mut VhdxFile) -> io::Result<(u64, String)> {
    let mut data: Vec<u8> = vec![0; 35891];
    let mut md5_context: Md5Context = Md5Context::new();
    let mut media_offset: u64 = 0;

    while let Ok(read_count) = file.read(&mut data) {
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

fn open_file(location: &str) -> io::Result<VhdxFile> {
    let mut file: VhdxFile = VhdxFile::new();

    let data_stream: DataStreamReference = open_os_data_stream(location)?;
    file.read_data_stream(&data_stream)?;

    Ok(file)
}

#[test]
fn read_media_fixed() -> io::Result<()> {
    let mut file: VhdxFile = open_file("../test_data/vhdx/ntfs-parent.vhdx")?;

    let (media_offset, md5_hash): (u64, String) = read_media_from_file(&mut file)?;
    assert_eq!(media_offset, file.media_size);
    assert_eq!(md5_hash.as_str(), "75537374a81c40e51e6a4b812b36ce89");

    Ok(())
}

#[test]
fn read_media_dynamic() -> io::Result<()> {
    let mut file: VhdxFile = open_file("../test_data/vhdx/ntfs-dynamic.vhdx")?;

    let (media_offset, md5_hash): (u64, String) = read_media_from_file(&mut file)?;
    assert_eq!(media_offset, file.media_size);
    assert_eq!(md5_hash.as_str(), "20158534070142d63ee02c9ad1a9d87e");

    Ok(())
}

#[test]
fn read_media_sparse_dynamic() -> io::Result<()> {
    let mut file: VhdxFile = open_file("../test_data/vhdx/ext2.vhdx")?;

    let (media_offset, md5_hash): (u64, String) = read_media_from_file(&mut file)?;
    assert_eq!(media_offset, file.media_size);
    assert_eq!(md5_hash.as_str(), "b1760d0b35a512ef56970df4e6f8c5d6");

    Ok(())
}

#[test]
fn read_media_differential() -> io::Result<()> {
    let mut parent_file: VhdxFile = VhdxFile::new();

    let data_stream: DataStreamReference =
        open_os_data_stream("../test_data/vhdx/ntfs-parent.vhdx")?;
    parent_file.read_data_stream(&data_stream)?;

    let mut file: VhdxFile = VhdxFile::new();

    let data_stream: DataStreamReference =
        open_os_data_stream("../test_data/vhdx/ntfs-differential.vhdx")?;
    file.read_data_stream(&data_stream)?;

    file.set_parent(&Arc::new(RwLock::new(parent_file)))?;

    let (media_offset, md5_hash): (u64, String) = read_media_from_file(&mut file)?;
    assert_eq!(media_offset, file.media_size);
    assert_eq!(md5_hash.as_str(), "a25df0058eecd8aa1975a68eeaa0e178");

    Ok(())
}
