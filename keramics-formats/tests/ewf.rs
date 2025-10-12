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
use keramics_core::{DataStream, ErrorTrace};
use keramics_formats::ewf::EwfImage;
use keramics_formats::{FileResolverReference, PathComponent, open_os_file_resolver};
use keramics_hashes::{DigestHashContext, Md5Context};

fn read_media_from_image(image: &mut EwfImage) -> Result<(u64, String), ErrorTrace> {
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
                        "Unable to read from EWF image at offset {} (0x{:08x})",
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

fn open_image(base_path: &PathBuf, file_name: &str) -> Result<EwfImage, ErrorTrace> {
    let file_resolver: FileResolverReference = match open_os_file_resolver(base_path) {
        Ok(data_stream) => data_stream,
        Err(error) => {
            return Err(keramics_core::error_trace_new_with_error!(
                "Unable to open file resolver",
                error
            ));
        }
    };
    let mut image: EwfImage = EwfImage::new();

    let path_component: PathComponent = PathComponent::from(file_name);
    match image.open(&file_resolver, &path_component) {
        Ok(_) => {}
        Err(mut error) => {
            keramics_core::error_trace_add_frame!(error, "Unable to open EWF image");
            return Err(error);
        }
    }
    Ok(image)
}

#[test]
fn read_media() -> Result<(), ErrorTrace> {
    let path_buf: PathBuf = PathBuf::from("../test_data/ewf");
    let mut image: EwfImage = open_image(&path_buf, "ext2.E01")?;

    let (media_offset, md5_hash): (u64, String) = read_media_from_image(&mut image)?;
    assert_eq!(media_offset, image.media_size);
    assert_eq!(md5_hash.as_str(), "b1760d0b35a512ef56970df4e6f8c5d6");

    Ok(())
}
