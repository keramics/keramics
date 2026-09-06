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
use std::sync::{Arc, RwLock};

use keramics_core::{DataStreamReference, ErrorTrace, open_os_data_stream};
use keramics_formats::RangeStream;
use keramics_formats::bde::{BdeCredential, BdeEncryptedVolume};
use keramics_formats::vhd::VhdFile;

mod util;

use util::read_data_stream;

fn open_encrypted_volume(path: &PathBuf) -> Result<BdeEncryptedVolume, ErrorTrace> {
    let os_data_stream: DataStreamReference = match open_os_data_stream(path) {
        Ok(data_stream) => data_stream,
        Err(error) => {
            return Err(keramics_core::error_trace_new_with_error!(
                "Unable to open data stream",
                error
            ));
        }
    };
    let mut vhd_file: VhdFile = VhdFile::new();

    match vhd_file.read_data_stream(&os_data_stream) {
        Ok(_) => {}
        Err(error) => {
            return Err(keramics_core::error_trace_new_with_error!(
                "Unable to open VHD file",
                error
            ));
        }
    }
    let vhd_data_stream: DataStreamReference = match vhd_file.get_data_stream() {
        Some(data_stream) => data_stream,
        None => {
            return Err(keramics_core::error_trace_new!("Missing VHD data stream"));
        }
    };
    let data_stream: DataStreamReference = Arc::new(RwLock::new(RangeStream::new(
        &vhd_data_stream,
        65536,
        65994752,
    )));
    let mut encrypted_volume: BdeEncryptedVolume = BdeEncryptedVolume::new();

    match encrypted_volume.read_data_stream(&data_stream) {
        Ok(_) => {}
        Err(mut error) => {
            keramics_core::error_trace_add_frame!(
                error,
                "Unable to read BDE encrypted volume from data stream"
            );
            return Err(error);
        }
    }
    Ok(encrypted_volume)
}

#[test]
fn read_encrypted_volume() -> Result<(), ErrorTrace> {
    let path_buf: PathBuf = PathBuf::from("../test_data/bde/bde_aes128.vhd");
    let mut encrypted_volume: BdeEncryptedVolume = open_encrypted_volume(&path_buf)?;
    let credentials: Vec<BdeCredential> = vec![BdeCredential::Passphrase(b"KeRaMiCs".to_vec())];
    encrypted_volume.unlock(&credentials)?;

    let data_stream: DataStreamReference = encrypted_volume.get_data_stream().unwrap();

    let (volume_offset, md5_hash): (u64, String) = read_data_stream(&data_stream)?;
    // let (volume_offset, md5_hash): (u64, String) = util::read_data_stream_with_output_file(&data_stream)?;

    assert_eq!(volume_offset, encrypted_volume.get_volume_size());
    assert_eq!(md5_hash.as_str(), "95c4b4e14b211ef9d1372ba47ed99dc8");

    Ok(())
}
