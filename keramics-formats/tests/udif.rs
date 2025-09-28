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

use keramics_core::formatters::format_as_string;
use keramics_core::{DataStreamReference, open_os_data_stream};
use keramics_formats::udif::UdifFile;
use keramics_hashes::{DigestHashContext, Md5Context};

fn read_media_from_file(file: &mut UdifFile) -> io::Result<(u64, String)> {
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

fn open_file(path: &str) -> io::Result<UdifFile> {
    let mut file: UdifFile = UdifFile::new();

    let data_stream: DataStreamReference = open_os_data_stream(path)?;
    file.read_data_stream(&data_stream)?;

    Ok(file)
}

#[test]
fn read_media_adc_compressed() -> io::Result<()> {
    let mut file: UdifFile = open_file("../test_data/udif/hfsplus_adc.dmg")?;

    let (media_offset, md5_hash): (u64, String) = read_media_from_file(&mut file)?;
    assert_eq!(media_offset, file.media_size);
    assert_eq!(md5_hash.as_str(), "08c32fd5d0fc1c2274d1c2d34185312a");

    Ok(())
}

#[test]
fn read_media_bzip2_compressed() -> io::Result<()> {
    let mut file: UdifFile = open_file("../test_data/udif/hfsplus_bzip2.dmg")?;

    let (media_offset, md5_hash): (u64, String) = read_media_from_file(&mut file)?;
    assert_eq!(media_offset, file.media_size);
    assert_eq!(md5_hash.as_str(), "7ec785450bbc17de417be373fd5d2159");

    Ok(())
}

#[test]
fn read_media_lzfse_compressed() -> io::Result<()> {
    let mut file: UdifFile = open_file("../test_data/udif/hfsplus_lzfse.dmg")?;

    let (media_offset, md5_hash): (u64, String) = read_media_from_file(&mut file)?;
    assert_eq!(media_offset, file.media_size);
    assert_eq!(md5_hash.as_str(), "c2c160c788676641725fd1a4b8da733b");

    Ok(())
}

#[test]
fn read_media_zlib_compressed() -> io::Result<()> {
    let mut file: UdifFile = open_file("../test_data/udif/hfsplus_zlib.dmg")?;

    let (media_offset, md5_hash): (u64, String) = read_media_from_file(&mut file)?;
    assert_eq!(media_offset, file.media_size);
    assert_eq!(md5_hash.as_str(), "399bfcc39637bde7e43eb86fcc8565ae");

    Ok(())
}
