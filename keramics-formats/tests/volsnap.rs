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

use std::path::{MAIN_SEPARATOR_STR, PathBuf};
use std::sync::{Arc, RwLock};

use keramics_core::{DataStreamReference, ErrorTrace, open_os_data_stream};
use keramics_formats::vhd::VhdFile;
use keramics_formats::volsnap::{VolsnapShadowStorage, VolsnapSnapshot};
use keramics_formats::{FileResolver, FileResolverReference, PathComponent, RangeStream};

mod util;

use util::read_data_stream;

struct TestFileResolver {
    /// Base path.
    base_path: PathBuf,
}

impl TestFileResolver {
    /// Creates a new file resolver.
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    /// Retrieves a path based on the base path and path components.
    fn get_path(&self, path_components: &[PathComponent]) -> PathBuf {
        let mut path_buf: PathBuf = self.base_path.clone();

        for path_component in path_components.iter() {
            match path_component {
                PathComponent::ByteString(byte_string) => {
                    path_buf.push(byte_string.to_string());
                }
                PathComponent::Current => path_buf.push("."),
                PathComponent::OsString(os_string) => path_buf.push(os_string),
                PathComponent::Parent => path_buf.push(".."),
                PathComponent::Root => path_buf.push(MAIN_SEPARATOR_STR),
                PathComponent::String(string) => path_buf.push(string),
                PathComponent::Ucs2String(ucs2_string) => {
                    path_buf.push(ucs2_string.to_string());
                }
                PathComponent::Utf16String(utf16_string) => {
                    path_buf.push(utf16_string.to_string());
                }
            }
        }
        path_buf
    }
}

impl FileResolver for TestFileResolver {
    /// Retrieves a data stream with the specified path.
    fn get_data_stream(
        &self,
        path_components: &[PathComponent],
    ) -> Result<Option<DataStreamReference>, ErrorTrace> {
        let path_buf: PathBuf = self.get_path(path_components);

        let os_data_stream: DataStreamReference = match open_os_data_stream(&path_buf) {
            Ok(data_stream) => data_stream,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!("Unable to open file: {}", path_buf.display())
                );
                return Err(error);
            }
        };
        let mut vhd_file: VhdFile = VhdFile::new();

        match vhd_file.read_data_stream(&os_data_stream) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open VHD file");
                return Err(error);
            }
        }
        let vhd_data_stream: DataStreamReference = match vhd_file.get_data_stream() {
            Some(data_stream) => data_stream,
            None => {
                return Err(keramics_core::error_trace_new!("Missing data stream"));
            }
        };
        Ok(Some(Arc::new(RwLock::new(RangeStream::new(
            &vhd_data_stream,
            65536,
            133103616,
        )))))
    }
}

fn open_shadow_storage() -> Result<VolsnapShadowStorage, ErrorTrace> {
    let mut shadow_storage: VolsnapShadowStorage = VolsnapShadowStorage::new();

    let path_buf: PathBuf = PathBuf::from("../test_data/volsnap");
    let file_resolver: FileResolverReference =
        FileResolverReference::new(Box::new(TestFileResolver::new(path_buf)));

    let file_names: [PathComponent; 1] = [PathComponent::from("volsnap.vhd")];
    shadow_storage.open(&file_resolver, &file_names)?;

    Ok(shadow_storage)
}

#[test]
fn read_snapshot1() -> Result<(), ErrorTrace> {
    let shadow_storage: VolsnapShadowStorage = open_shadow_storage()?;

    let snapshot: VolsnapSnapshot = shadow_storage.get_snapshot_by_index(0)?;

    let data_stream: DataStreamReference = snapshot.get_data_stream().unwrap();

    let (volume_offset, md5_hash): (u64, String) = read_data_stream(&data_stream)?;

    let volume_size: u64 = *snapshot.get_size().unwrap();
    assert_eq!(volume_offset, volume_size);
    assert_eq!(md5_hash.as_str(), "7046d22cc29ca53d4aedbb3118eca044");

    Ok(())
}

#[test]
fn read_snapshot2() -> Result<(), ErrorTrace> {
    let shadow_storage: VolsnapShadowStorage = open_shadow_storage()?;

    let snapshot: VolsnapSnapshot = shadow_storage.get_snapshot_by_index(1)?;

    let data_stream: DataStreamReference = snapshot.get_data_stream().unwrap();

    let (volume_offset, md5_hash): (u64, String) = read_data_stream(&data_stream)?;

    let volume_size: u64 = *snapshot.get_size().unwrap();
    assert_eq!(volume_offset, volume_size);
    assert_eq!(md5_hash.as_str(), "d6206ea68fd265de252e8133bed2e3fa");

    Ok(())
}
