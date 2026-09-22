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

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_formats::cdsaencr::CdsaEncrCredential;
use keramics_formats::udif::UdifImage;
use keramics_formats::{FileResolverReference, OsFileResolver, PathComponent};

mod util;

use util::read_data_stream;

fn open_image(base_path: &PathBuf, file_name: &str) -> Result<UdifImage, ErrorTrace> {
    let file_resolver: FileResolverReference =
        FileResolverReference::new(Box::new(OsFileResolver::new(base_path.clone())));
    let mut image: UdifImage = UdifImage::new();
    let path_component: PathComponent = PathComponent::from(file_name);

    match image.open(&file_resolver, &path_component) {
        Ok(_) => {}
        Err(mut error) => {
            keramics_core::error_trace_add_frame!(error, "Unable to open UDIF image");
            return Err(error);
        }
    }
    Ok(image)
}

#[test]
fn read_image_adc_compressed() -> Result<(), ErrorTrace> {
    let path_buf: PathBuf = PathBuf::from("../test_data/udif");
    let image: UdifImage = open_image(&path_buf, "hfsplus_adc.dmg")?;
    let data_stream: DataStreamReference = image.get_data_stream().unwrap();

    let (media_offset, md5_hash): (u64, String) = read_data_stream(&data_stream)?;

    assert_eq!(media_offset, image.get_media_size());
    assert_eq!(md5_hash.as_str(), "08c32fd5d0fc1c2274d1c2d34185312a");

    Ok(())
}

#[test]
fn read_image_aes128_encrypted_and_zlib_compressed() -> Result<(), ErrorTrace> {
    let path_buf: PathBuf = PathBuf::from("../test_data/udif");
    let mut image: UdifImage = open_image(&path_buf, "hfsplus_zlib_aes128.dmg")?;

    // Using the key data to bypass key derivation.
    let credentials: Vec<CdsaEncrCredential> = vec![CdsaEncrCredential::KeyData {
        identifier: vec![
            0x9d, 0x5f, 0x93, 0xb2, 0x71, 0x67, 0x40, 0x62, 0xa2, 0x88, 0x8e, 0x3d, 0x68, 0xb3,
            0x78, 0x19,
        ],
        data: vec![
            0x64, 0xde, 0x30, 0x9a, 0xf1, 0xca, 0xdf, 0x7b, 0xee, 0x99, 0x8a, 0xb9, 0x1d, 0x39,
            0xcc, 0x30, 0x64, 0xde, 0x30, 0x9a, 0xf1, 0xca, 0xdf, 0x7b, 0xee, 0x99, 0x8a, 0xb9,
            0x1d, 0x39, 0xcc, 0x30, 0x18, 0xd4, 0x3f, 0xfb,
        ],
    }];
    image.unlock(&credentials)?;

    let data_stream: DataStreamReference = image.get_data_stream().unwrap();

    let (media_offset, md5_hash): (u64, String) = read_data_stream(&data_stream)?;

    assert_eq!(media_offset, image.get_media_size());
    assert_eq!(md5_hash.as_str(), "399bfcc39637bde7e43eb86fcc8565ae");

    Ok(())
}

#[test]
fn read_image_aes256_encrypted() -> Result<(), ErrorTrace> {
    let path_buf: PathBuf = PathBuf::from("../test_data/udif");
    let mut image: UdifImage = open_image(&path_buf, "hfsplus_aes256.dmg")?;

    // Using the key data to bypass key derivation.
    let credentials: Vec<CdsaEncrCredential> = vec![CdsaEncrCredential::KeyData {
        identifier: vec![
            0x6d, 0xde, 0x70, 0x6c, 0x61, 0xd2, 0x45, 0xff, 0x90, 0x46, 0xc8, 0x6b, 0x39, 0x12,
            0xbf, 0xeb,
        ],
        data: vec![
            0x76, 0x7e, 0xbc, 0xb0, 0x25, 0xb9, 0x31, 0xed, 0xf9, 0x53, 0xa7, 0xcd, 0x50, 0xc3,
            0xb2, 0xbb, 0x3a, 0xd4, 0x51, 0x6a, 0x83, 0xf4, 0x82, 0x13, 0x2b, 0xfc, 0x3d, 0x99,
            0x4b, 0x4f, 0x51, 0x36, 0x28, 0x68, 0xbd, 0x1c, 0x71, 0xcf, 0x29, 0x36, 0x8d, 0xe2,
            0xc7, 0xfc, 0x50, 0x2a, 0xd7, 0x1e, 0x6e, 0xee, 0xdd, 0x24,
        ],
    }];
    image.unlock(&credentials)?;

    let data_stream: DataStreamReference = image.get_data_stream().unwrap();

    let (media_offset, md5_hash): (u64, String) = read_data_stream(&data_stream)?;

    assert_eq!(media_offset, image.get_media_size());
    assert_eq!(md5_hash.as_str(), "399bfcc39637bde7e43eb86fcc8565ae");

    Ok(())
}

#[test]
fn read_image_bzip2_compressed() -> Result<(), ErrorTrace> {
    let path_buf: PathBuf = PathBuf::from("../test_data/udif");
    let image: UdifImage = open_image(&path_buf, "hfsplus_bzip2.dmg")?;
    let data_stream: DataStreamReference = image.get_data_stream().unwrap();

    let (media_offset, md5_hash): (u64, String) = read_data_stream(&data_stream)?;

    assert_eq!(media_offset, image.get_media_size());
    assert_eq!(md5_hash.as_str(), "7ec785450bbc17de417be373fd5d2159");

    Ok(())
}

#[test]
fn read_image_lzfse_compressed() -> Result<(), ErrorTrace> {
    let path_buf: PathBuf = PathBuf::from("../test_data/udif");
    let image: UdifImage = open_image(&path_buf, "hfsplus_lzfse.dmg")?;
    let data_stream: DataStreamReference = image.get_data_stream().unwrap();

    let (media_offset, md5_hash): (u64, String) = read_data_stream(&data_stream)?;

    assert_eq!(media_offset, image.get_media_size());
    assert_eq!(md5_hash.as_str(), "c2c160c788676641725fd1a4b8da733b");

    Ok(())
}

#[test]
fn read_image_with_resource_fork() -> Result<(), ErrorTrace> {
    let path_buf: PathBuf = PathBuf::from("../test_data/udif");
    let image: UdifImage = open_image(&path_buf, "hfsplus_rsrc.dmg")?;
    let data_stream: DataStreamReference = image.get_data_stream().unwrap();

    let (media_offset, md5_hash): (u64, String) = read_data_stream(&data_stream)?;

    assert_eq!(media_offset, image.get_media_size());
    assert_eq!(md5_hash.as_str(), "399bfcc39637bde7e43eb86fcc8565ae");

    Ok(())
}

#[test]
fn read_image_with_segments() -> Result<(), ErrorTrace> {
    let path_buf: PathBuf = PathBuf::from("../test_data/udif");
    let image: UdifImage = open_image(&path_buf, "hfsplus_segments.dmg")?;
    let data_stream: DataStreamReference = image.get_data_stream().unwrap();

    let (media_offset, md5_hash): (u64, String) = read_data_stream(&data_stream)?;

    assert_eq!(media_offset, image.get_media_size());
    assert_eq!(md5_hash.as_str(), "399bfcc39637bde7e43eb86fcc8565ae");

    Ok(())
}

#[test]
fn read_image_zlib_compressed() -> Result<(), ErrorTrace> {
    let path_buf: PathBuf = PathBuf::from("../test_data/udif");
    let image: UdifImage = open_image(&path_buf, "hfsplus_zlib.dmg")?;
    let data_stream: DataStreamReference = image.get_data_stream().unwrap();

    let (media_offset, md5_hash): (u64, String) = read_data_stream(&data_stream)?;

    assert_eq!(media_offset, image.get_media_size());
    assert_eq!(md5_hash.as_str(), "399bfcc39637bde7e43eb86fcc8565ae");

    Ok(())
}

#[test]
fn read_image_zlib_compressed_with_segments() -> Result<(), ErrorTrace> {
    let path_buf: PathBuf = PathBuf::from("../test_data/udif");
    let image: UdifImage = open_image(&path_buf, "hfsplus_zlib_segments.dmg")?;
    let data_stream: DataStreamReference = image.get_data_stream().unwrap();

    let (media_offset, md5_hash): (u64, String) = read_data_stream(&data_stream)?;

    assert_eq!(media_offset, image.get_media_size());
    assert_eq!(md5_hash.as_str(), "399bfcc39637bde7e43eb86fcc8565ae");

    Ok(())
}
