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

use keramics_core::{DataStreamReference, ErrorTrace, open_os_data_stream};
use keramics_formats::vhd::VhdFile;

mod util;

use util::read_data_stream;

fn open_file(path: &PathBuf) -> Result<VhdFile, ErrorTrace> {
    let data_stream: DataStreamReference = match open_os_data_stream(path) {
        Ok(data_stream) => data_stream,
        Err(error) => {
            return Err(keramics_core::error_trace_new_with_error!(
                "Unable to open data stream",
                error
            ));
        }
    };
    let mut file: VhdFile = VhdFile::new();

    match file.read_data_stream(&data_stream) {
        Ok(_) => {}
        Err(mut error) => {
            keramics_core::error_trace_add_frame!(
                error,
                "Unable to read VHD file from data stream"
            );
            return Err(error);
        }
    }
    Ok(file)
}

#[test]
fn read_media_fixed() -> Result<(), ErrorTrace> {
    let path_buf: PathBuf = PathBuf::from("../test_data/vhd/ntfs-parent.vhd");
    let file: VhdFile = open_file(&path_buf)?;
    let data_stream: DataStreamReference = file.get_data_stream().unwrap();

    let (media_offset, md5_hash): (u64, String) = read_data_stream(&data_stream)?;

    assert_eq!(media_offset, file.get_media_size());
    assert_eq!(md5_hash.as_str(), "acb42a740c63c1f72e299463375751c8");

    Ok(())
}

#[test]
fn read_media_dynamic() -> Result<(), ErrorTrace> {
    let path_buf: PathBuf = PathBuf::from("../test_data/vhd/ntfs-dynamic.vhd");
    let file: VhdFile = open_file(&path_buf)?;
    let data_stream: DataStreamReference = file.get_data_stream().unwrap();

    let (media_offset, md5_hash): (u64, String) = read_data_stream(&data_stream)?;

    assert_eq!(media_offset, file.get_media_size());
    assert_eq!(md5_hash.as_str(), "4ce30a0c21dd037023a5692d85ade033");

    Ok(())
}

#[test]
fn read_media_sparse_dynamic() -> Result<(), ErrorTrace> {
    let path_buf: PathBuf = PathBuf::from("../test_data/vhd/ext2.vhd");
    let file: VhdFile = open_file(&path_buf)?;
    let data_stream: DataStreamReference = file.get_data_stream().unwrap();

    let (media_offset, md5_hash): (u64, String) = read_data_stream(&data_stream)?;

    assert_eq!(media_offset, file.get_media_size());
    // Note that the VHD has 18432 bytes of additional storage media data due to the image
    // creation process.
    assert_eq!(md5_hash.as_str(), "a30f111f411d3f3d567b13f0c909e58c");

    Ok(())
}

#[test]
fn read_media_differential() -> Result<(), ErrorTrace> {
    let path_buf: PathBuf = PathBuf::from("../test_data/vhd/ntfs-parent.vhd");
    let parent_file: VhdFile = open_file(&path_buf)?;

    let path_buf: PathBuf = PathBuf::from("../test_data/vhd/ntfs-differential.vhd");
    let mut file: VhdFile = open_file(&path_buf)?;
    file.set_parent(&Arc::new(parent_file))?;

    let data_stream: DataStreamReference = file.get_data_stream().unwrap();

    let (media_offset, md5_hash): (u64, String) = read_data_stream(&data_stream)?;

    assert_eq!(media_offset, file.get_media_size());
    assert_eq!(md5_hash.as_str(), "4241cbc76e0e17517fb564238edbe415");

    Ok(())
}
