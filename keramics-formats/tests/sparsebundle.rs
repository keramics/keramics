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
use keramics_formats::sparsebundle::SparseBundleImage;
use keramics_formats::{FileResolverReference, OsFileResolver, PathComponent};

mod util;

use util::read_data_stream;

fn open_image(base_path: &PathBuf) -> Result<SparseBundleImage, ErrorTrace> {
    let file_resolver: FileResolverReference =
        FileResolverReference::new(Box::new(OsFileResolver::new(base_path.clone())));
    let mut image: SparseBundleImage = SparseBundleImage::new();

    let file_name: PathComponent = PathComponent::from("Info.plist");
    match image.open(&file_resolver, &file_name) {
        Ok(_) => {}
        Err(mut error) => {
            keramics_core::error_trace_add_frame!(error, "Unable to open sparsebundle image");
            return Err(error);
        }
    }
    Ok(image)
}

#[test]
fn read_image() -> Result<(), ErrorTrace> {
    let path_buf: PathBuf = PathBuf::from("../test_data/sparsebundle/hfsplus.sparsebundle");
    let image: SparseBundleImage = open_image(&path_buf)?;
    let data_stream: DataStreamReference = image.get_data_stream().unwrap();

    let (media_offset, md5_hash): (u64, String) = read_data_stream(&data_stream)?;

    assert_eq!(media_offset, image.get_media_size());
    assert_eq!(md5_hash.as_str(), "7adf013daec71e509669a9315a6a173c");

    Ok(())
}

#[test]
fn read_image_encrypted() -> Result<(), ErrorTrace> {
    let path_buf: PathBuf = PathBuf::from("../test_data/sparsebundle/hfsplus_aes128.sparsebundle");
    let mut image: SparseBundleImage = open_image(&path_buf)?;

    // Using the key data to bypass key derivation.
    let credentials: Vec<CdsaEncrCredential> = vec![CdsaEncrCredential::KeyData {
        identifier: vec![
            0xc3, 0x37, 0xc7, 0x68, 0x2e, 0x12, 0x40, 0x07, 0xa1, 0xf0, 0x3f, 0x81, 0x2c, 0x96,
            0xda, 0xef,
        ],
        data: vec![
            0xef, 0x96, 0x09, 0x6a, 0x75, 0x62, 0x2e, 0x15, 0xdd, 0xc5, 0x23, 0x49, 0x90, 0x2a,
            0xc1, 0x24, 0xef, 0x96, 0x09, 0x6a, 0x75, 0x62, 0x2e, 0x15, 0xdd, 0xc5, 0x23, 0x49,
            0x90, 0x2a, 0xc1, 0x24, 0x00, 0x00, 0x00, 0x00,
        ],
    }];
    image.unlock(&credentials)?;

    let data_stream: DataStreamReference = image.get_data_stream().unwrap();

    let (media_offset, md5_hash): (u64, String) = read_data_stream(&data_stream)?;

    assert_eq!(media_offset, image.get_media_size());
    assert_eq!(md5_hash.as_str(), "2d568b020506121467d1d97bcc024f68");

    Ok(())
}
