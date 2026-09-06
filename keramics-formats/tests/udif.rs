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
use keramics_formats::{FileResolverReference, PathComponent, open_os_file_resolver};

mod util;

use util::read_data_stream;

fn open_image(base_path: &PathBuf, file_name: &str) -> Result<UdifImage, ErrorTrace> {
    let file_resolver: FileResolverReference = match open_os_file_resolver(base_path) {
        Ok(data_stream) => data_stream,
        Err(error) => {
            return Err(keramics_core::error_trace_new_with_error!(
                "Unable to open file resolver",
                error
            ));
        }
    };
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
    let credentials: Vec<CdsaEncrCredential> =
        vec![CdsaEncrCredential::Passphrase(b"KeRaMiCs".to_vec())];
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
    let credentials: Vec<CdsaEncrCredential> =
        vec![CdsaEncrCredential::Passphrase(b"KeRaMiCs".to_vec())];
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
