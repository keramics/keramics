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
use keramics_core::{open_os_file_resolver, FileResolverReference};
use keramics_formats::sparsebundle::SparseBundleImage;
use keramics_hashes::{DigestHashContext, Md5Context};

fn read_media_from_image(image: &mut SparseBundleImage) -> io::Result<(u64, String)> {
    let mut data: Vec<u8> = vec![0; 35891];
    let mut md5_context: Md5Context = Md5Context::new();
    let mut media_offset: u64 = 0;

    while let Ok(read_count) = image.read(&mut data) {
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

#[test]
fn read_media() -> io::Result<()> {
    let mut image = SparseBundleImage::new();

    let file_resolver: FileResolverReference =
        open_os_file_resolver("../test_data/sparsebundle/hfsplus.sparsebundle")?;
    image.open(&file_resolver)?;

    let (media_offset, md5_hash): (u64, String) = read_media_from_image(&mut image)?;
    assert_eq!(media_offset, image.media_size);
    assert_eq!(md5_hash.as_str(), "7adf013daec71e509669a9315a6a173c");

    Ok(())
}
