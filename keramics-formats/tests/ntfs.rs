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

use keramics_core::{DataStreamReference, ErrorTrace, open_os_data_stream};
use keramics_formats::Path;
use keramics_formats::ntfs::constants::{NTFS_ATTRIBUTE_TYPE_DATA, NTFS_ATTRIBUTE_TYPE_INDEX_ROOT};
use keramics_formats::ntfs::{NtfsAttribute, NtfsFileEntry, NtfsFileSystem};
use keramics_types::Ucs2String;

mod util;

use util::read_data_stream;

fn read_path(file_system: &NtfsFileSystem, path_string: &str) -> Result<(u64, String), ErrorTrace> {
    let path: Path = Path::from(path_string);
    let result: Option<NtfsFileEntry> = match file_system.get_file_entry_by_path(&path) {
        Ok(result) => result,
        Err(mut error) => {
            keramics_core::error_trace_add_frame!(
                error,
                format!("Unable to retrieve file entry: {}", path_string)
            );
            return Err(error);
        }
    };
    let file_entry: NtfsFileEntry = match result {
        Some(file_entry) => file_entry,
        None => {
            return Err(keramics_core::error_trace_new!(format!(
                "Missing data stream for file entry: {}",
                path_string
            )));
        }
    };
    let data_stream: DataStreamReference = file_entry.get_data_stream()?.unwrap();

    read_data_stream(&data_stream)
}

fn open_file_system(path: &PathBuf) -> Result<NtfsFileSystem, ErrorTrace> {
    let data_stream: DataStreamReference = match open_os_data_stream(path) {
        Ok(data_stream) => data_stream,
        Err(mut error) => {
            keramics_core::error_trace_add_frame!(error, "Unable to open data stream");
            return Err(error);
        }
    };
    let mut file_system: NtfsFileSystem = NtfsFileSystem::new();

    match file_system.read_data_stream(&data_stream) {
        Ok(_) => {}
        Err(mut error) => {
            keramics_core::error_trace_add_frame!(
                error,
                "Unable to read NTFS file system from data stream"
            );
            return Err(error);
        }
    };
    Ok(file_system)
}

#[test]
fn read_ntfs_empty_file() -> Result<(), ErrorTrace> {
    let path_buf: PathBuf = PathBuf::from("../test_data/ntfs/ntfs.raw");
    let file_system: NtfsFileSystem = open_file_system(&path_buf)?;

    let (offset, md5_hash): (u64, String) = read_path(&file_system, "/emptyfile")?;
    assert_eq!(offset, 0);
    assert_eq!(md5_hash.as_str(), "d41d8cd98f00b204e9800998ecf8427e");

    Ok(())
}

#[test]
fn read_ntfs_file_regular() -> Result<(), ErrorTrace> {
    let path_buf: PathBuf = PathBuf::from("../test_data/ntfs/ntfs.raw");
    let file_system: NtfsFileSystem = open_file_system(&path_buf)?;

    let (offset, md5_hash): (u64, String) = read_path(&file_system, "/testdir1/TestFile2")?;
    assert_eq!(offset, 11358);
    assert_eq!(md5_hash.as_str(), "3b83ef96387f14655fc854ddc3c6bd57");

    Ok(())
}

#[test]
fn test_read_attribute_data_stream() -> Result<(), ErrorTrace> {
    let path_buf: PathBuf = PathBuf::from("../test_data/ntfs/ntfs.raw");
    let file_system: NtfsFileSystem = open_file_system(&path_buf)?;

    let path: Path = Path::from("/testdir1/TestFile2");
    let file_entry: NtfsFileEntry = file_system
        .get_file_entry_by_path(&path)?
        .expect("Missing file entry");

    let direct_stream: DataStreamReference = file_entry
        .get_data_stream()?
        .expect("Missing direct data stream");
    let (direct_offset, direct_md5): (u64, String) = read_data_stream(&direct_stream)?;
    assert_eq!(direct_offset, 11358);
    assert_eq!(direct_md5.as_str(), "3b83ef96387f14655fc854ddc3c6bd57");

    let number_of_attributes: usize = file_entry.get_number_of_attributes();
    let mut data_stream_from_attr: Option<DataStreamReference> = None;

    for index in 0..number_of_attributes {
        let attribute: NtfsAttribute = file_entry.get_attribute_by_index(index)?;
        if attribute.get_attribute_type() == NTFS_ATTRIBUTE_TYPE_DATA
            && attribute.get_name().is_none()
        {
            let stream: DataStreamReference = attribute.get_data_stream()?;
            data_stream_from_attr = Some(stream);
            break;
        }
    }

    let attr_stream: DataStreamReference =
        data_stream_from_attr.expect("Missing $DATA attribute on file entry");
    let (attr_offset, attr_md5): (u64, String) = read_data_stream(&attr_stream)?;
    assert_eq!(attr_offset, direct_offset);
    assert_eq!(attr_md5, direct_md5);

    Ok(())
}

#[test]
fn test_read_attribute_index_root() -> Result<(), ErrorTrace> {
    let path_buf: PathBuf = PathBuf::from("../test_data/ntfs/ntfs.raw");
    let file_system: NtfsFileSystem = open_file_system(&path_buf)?;

    let path: Path = Path::from("/testdir1");
    let file_entry: NtfsFileEntry = file_system
        .get_file_entry_by_path(&path)?
        .expect("Missing directory entry");

    let number_of_attributes: usize = file_entry.get_number_of_attributes();
    let mut index_root_stream: Option<DataStreamReference> = None;
    let expected_name: Ucs2String = Ucs2String::from("$I30");

    for index in 0..number_of_attributes {
        let attribute: NtfsAttribute = file_entry.get_attribute_by_index(index)?;
        if attribute.get_attribute_type() == NTFS_ATTRIBUTE_TYPE_INDEX_ROOT {
            assert_eq!(attribute.get_name(), Some(&expected_name));
            let stream: DataStreamReference = attribute.get_data_stream()?;
            index_root_stream = Some(stream);
            break;
        }
    }

    let stream: DataStreamReference =
        index_root_stream.expect("Missing $INDEX_ROOT attribute on directory entry");
    let (index_root_size, _): (u64, String) = read_data_stream(&stream)?;
    assert!(index_root_size > 0);

    Ok(())
}

#[test]
fn test_read_attribute_allocated_data_stream() -> Result<(), ErrorTrace> {
    let path_buf: PathBuf = PathBuf::from("../test_data/ntfs/ntfs.raw");
    let file_system: NtfsFileSystem = open_file_system(&path_buf)?;

    let path: Path = Path::from("/testdir1/TestFile2");
    let file_entry: NtfsFileEntry = file_system
        .get_file_entry_by_path(&path)?
        .expect("Missing file entry");

    let number_of_attributes: usize = file_entry.get_number_of_attributes();
    let mut data_stream_logical: Option<DataStreamReference> = None;
    let mut data_stream_allocated: Option<DataStreamReference> = None;

    for index in 0..number_of_attributes {
        let attribute: NtfsAttribute = file_entry.get_attribute_by_index(index)?;
        if attribute.get_attribute_type() == NTFS_ATTRIBUTE_TYPE_DATA
            && attribute.get_name().is_none()
        {
            assert_eq!(attribute.get_allocated_data_size(), 12288);
            assert_eq!(attribute.get_data_size(), 11358);
            assert_eq!(attribute.get_valid_data_size(), 11358);

            let stream_logical: DataStreamReference = attribute.get_data_stream()?;
            data_stream_logical = Some(stream_logical);

            let stream_allocated: DataStreamReference = attribute.get_allocated_data_stream()?;
            data_stream_allocated = Some(stream_allocated);
            break;
        }
    }

    let stream_logical: DataStreamReference =
        data_stream_logical.expect("Missing $DATA attribute on file entry");
    let (read_size, md5): (u64, String) = read_data_stream(&stream_logical)?;
    assert_eq!(read_size, 11358);
    assert_eq!(md5.as_str(), "3b83ef96387f14655fc854ddc3c6bd57");

    let stream_allocated: DataStreamReference =
        data_stream_allocated.expect("Missing $DATA attribute on file entry");
    let (read_size_allocated, _): (u64, String) = read_data_stream(&stream_allocated)?;
    assert_eq!(read_size_allocated, 12288);

    Ok(())
}
