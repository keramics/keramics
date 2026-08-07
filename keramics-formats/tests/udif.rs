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

use keramics_core::formatters::format_as_string;
use keramics_core::{DataStream, ErrorTrace};
use keramics_formats::udif::UdifImage;
use keramics_formats::{FileResolverReference, PathComponent, open_os_file_resolver};
use keramics_hashes::{DigestHashContext, Md5Context};

fn read_media_from_image(image: &mut UdifImage) -> Result<(u64, String), ErrorTrace> {
    let mut data: Vec<u8> = vec![0; 35891];
    let mut md5_context: Md5Context = Md5Context::new();
    let mut media_offset: u64 = 0;

    loop {
        let read_count = match image.read(&mut data) {
            Ok(read_count) => read_count,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to read from UDIF image at offset {} (0x{:08x})",
                        media_offset, media_offset
                    )
                );
                return Err(error);
            }
        };
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
fn read_media_adc_compressed() -> Result<(), ErrorTrace> {
    let path_buf: PathBuf = PathBuf::from("../test_data/udif");
    let mut image: UdifImage = open_image(&path_buf, "hfsplus_adc.dmg")?;

    let (media_offset, md5_hash): (u64, String) = read_media_from_image(&mut image)?;
    assert_eq!(media_offset, image.get_media_size());
    assert_eq!(md5_hash.as_str(), "08c32fd5d0fc1c2274d1c2d34185312a");

    Ok(())
}

#[test]
fn read_media_bzip2_compressed() -> Result<(), ErrorTrace> {
    let path_buf: PathBuf = PathBuf::from("../test_data/udif");
    let mut image: UdifImage = open_image(&path_buf, "hfsplus_bzip2.dmg")?;

    let (media_offset, md5_hash): (u64, String) = read_media_from_image(&mut image)?;
    assert_eq!(media_offset, image.get_media_size());
    assert_eq!(md5_hash.as_str(), "7ec785450bbc17de417be373fd5d2159");

    Ok(())
}

#[test]
fn read_media_lzfse_compressed() -> Result<(), ErrorTrace> {
    let path_buf: PathBuf = PathBuf::from("../test_data/udif");
    let mut image: UdifImage = open_image(&path_buf, "hfsplus_lzfse.dmg")?;

    let (media_offset, md5_hash): (u64, String) = read_media_from_image(&mut image)?;
    assert_eq!(media_offset, image.get_media_size());
    assert_eq!(md5_hash.as_str(), "c2c160c788676641725fd1a4b8da733b");

    Ok(())
}

#[test]
fn read_media_with_resource_fork() -> Result<(), ErrorTrace> {
    let path_buf: PathBuf = PathBuf::from("../test_data/udif");
    let mut image: UdifImage = open_image(&path_buf, "hfsplus_rsrc.dmg")?;

    let (media_offset, md5_hash): (u64, String) = read_media_from_image(&mut image)?;
    assert_eq!(media_offset, image.get_media_size());
    assert_eq!(md5_hash.as_str(), "399bfcc39637bde7e43eb86fcc8565ae");

    Ok(())
}

#[test]
fn read_media_with_segments() -> Result<(), ErrorTrace> {
    let path_buf: PathBuf = PathBuf::from("../test_data/udif");
    let mut image: UdifImage = open_image(&path_buf, "hfsplus_segments.dmg")?;

    let (media_offset, md5_hash): (u64, String) = read_media_from_image(&mut image)?;
    assert_eq!(media_offset, image.get_media_size());
    assert_eq!(md5_hash.as_str(), "399bfcc39637bde7e43eb86fcc8565ae");

    Ok(())
}

#[test]
fn read_media_zlib_compressed() -> Result<(), ErrorTrace> {
    let path_buf: PathBuf = PathBuf::from("../test_data/udif");
    let mut image: UdifImage = open_image(&path_buf, "hfsplus_zlib.dmg")?;

    let (media_offset, md5_hash): (u64, String) = read_media_from_image(&mut image)?;
    assert_eq!(media_offset, image.get_media_size());
    assert_eq!(md5_hash.as_str(), "399bfcc39637bde7e43eb86fcc8565ae");

    Ok(())
}

#[test]
fn read_media_zlib_compressed_with_segments() -> Result<(), ErrorTrace> {
    let path_buf: PathBuf = PathBuf::from("../test_data/udif");
    let mut image: UdifImage = open_image(&path_buf, "hfsplus_zlib_segments.dmg")?;

    let (media_offset, md5_hash): (u64, String) = read_media_from_image(&mut image)?;
    assert_eq!(media_offset, image.get_media_size());
    assert_eq!(md5_hash.as_str(), "399bfcc39637bde7e43eb86fcc8565ae");

    Ok(())
}
