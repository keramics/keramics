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
use std::sync::Arc;

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_formats::pdi::{PdiImage, PdiImageLayer};
use keramics_formats::{FileResolverReference, open_os_file_resolver};

mod util;

use util::read_data_stream;

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
fn read_image() -> Result<(), ErrorTrace> {
    let path_buf: PathBuf = PathBuf::from("../test_data/pdi/hfsplus.hdd");
    let image: PdiImage = open_image(&path_buf)?;
    let number_of_layers: usize = image.get_number_of_layers();
    let image_layer: Arc<PdiImageLayer> = image.get_layer_by_index(number_of_layers - 1)?;
    let data_stream: DataStreamReference = image_layer.get_data_stream();

    let (media_offset, md5_hash): (u64, String) = read_data_stream(&data_stream)?;

    assert_eq!(media_offset, image.get_media_size());
    assert_eq!(md5_hash.as_str(), "ecaef634016fc699807cec47cef11dda");

    Ok(())
}
