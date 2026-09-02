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
use keramics_formats::ewf::EwfImage;
use keramics_formats::{FileResolverReference, PathComponent, open_os_file_resolver};

mod util;

use util::read_data_stream;

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
    let image: EwfImage = open_image(&path_buf, "ext2.E01")?;
    let data_stream: DataStreamReference = image.get_data_stream();

    let (media_offset, md5_hash): (u64, String) = read_data_stream(&data_stream)?;

    assert_eq!(media_offset, image.get_media_size());
    assert_eq!(md5_hash.as_str(), "b1760d0b35a512ef56970df4e6f8c5d6");

    Ok(())
}
