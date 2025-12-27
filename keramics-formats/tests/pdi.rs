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
use std::sync::{Arc, RwLock};

use keramics_core::formatters::format_as_string;
use keramics_core::{DataStream, ErrorTrace};
use keramics_formats::pdi::{PdiImage, PdiImageLayer};
use keramics_formats::{FileResolverReference, open_os_file_resolver};
use keramics_hashes::{DigestHashContext, Md5Context};

fn read_media_from_image_layer(
    image_layer: &mut PdiImageLayer,
) -> Result<(u64, String), ErrorTrace> {
    let mut data: Vec<u8> = vec![0; 35891];
    let mut md5_context: Md5Context = Md5Context::new();
    let mut media_offset: u64 = 0;

    loop {
        let read_count = match image_layer.read(&mut data) {
            Ok(read_count) => read_count,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to read from PDI image layer at offset {} (0x{:08x})",
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

fn open_image(base_path: &PathBuf) -> Result<PdiImage, ErrorTrace> {
    let file_resolver: FileResolverReference = match open_os_file_resolver(base_path) {
        Ok(data_stream) => data_stream,
        Err(error) => {
            return Err(keramics_core::error_trace_new_with_error!(
                "Unable to open file resolver",
                error
            ));
        }
    };
    let mut image: PdiImage = PdiImage::new();

    match image.open(&file_resolver) {
        Ok(_) => {}
        Err(mut error) => {
            keramics_core::error_trace_add_frame!(error, "Unable to open PDI image");
            return Err(error);
        }
    }
    Ok(image)
}

#[test]
fn read_media() -> Result<(), ErrorTrace> {
    let path_buf: PathBuf = PathBuf::from("../test_data/pdi/hfsplus.hdd");
    let image: PdiImage = open_image(&path_buf)?;

    let number_of_layers: usize = image.get_number_of_layers();

    let image_layer: Arc<RwLock<PdiImageLayer>> = image.get_layer_by_index(number_of_layers - 1)?;
    let (media_offset, md5_hash): (u64, String) = match image_layer.write() {
        Ok(mut pdi_image_layer) => read_media_from_image_layer(&mut pdi_image_layer)?,
        Err(_) => {
            return Err(keramics_core::error_trace_new!(
                "Unable to obtain write lock on image layer"
            ));
        }
    };
    assert_eq!(media_offset, image.media_size);
    assert_eq!(md5_hash.as_str(), "ecaef634016fc699807cec47cef11dda");

    Ok(())
}
