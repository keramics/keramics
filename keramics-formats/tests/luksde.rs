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
use keramics_formats::luksde::{LuksCredential, LuksEncryptedVolume};

mod util;

use util::read_data_stream;

fn open_encrypted_volume(path: &PathBuf) -> Result<LuksEncryptedVolume, ErrorTrace> {
    let data_stream: DataStreamReference = match open_os_data_stream(path) {
        Ok(data_stream) => data_stream,
        Err(mut error) => {
            keramics_core::error_trace_add_frame!(error, "Unable to open data stream");
            return Err(error);
        }
    };
    let mut encrypted_volume: LuksEncryptedVolume = LuksEncryptedVolume::new();

    match encrypted_volume.read_data_stream(&data_stream) {
        Ok(_) => {}
        Err(mut error) => {
            keramics_core::error_trace_add_frame!(
                error,
                "Unable to read LUKS encrypted volume from data stream"
            );
            return Err(error);
        }
    }
    Ok(encrypted_volume)
}

#[test]
fn read_encrypted_volume() -> Result<(), ErrorTrace> {
    let path_buf: PathBuf = PathBuf::from("../test_data/luksde/luks1.raw");
    let mut encrypted_volume: LuksEncryptedVolume = open_encrypted_volume(&path_buf)?;

    // Using the key data to bypass key derivation.
    let credentials: Vec<LuksCredential> = vec![LuksCredential::KeyData {
        identifier: vec![
            0x20, 0xbc, 0x27, 0x95, 0x63, 0xf3, 0x4d, 0xc4, 0x80, 0xd8, 0x07, 0x91, 0x19, 0x13,
            0xa0, 0x31,
        ],
        data: vec![
            0xd2, 0xc4, 0x0d, 0xbb, 0xe4, 0xba, 0x5c, 0xe7, 0xf7, 0x85, 0xa4, 0x8a, 0xfc, 0x34,
            0xfa, 0x40, 0xfa, 0x90, 0x86, 0x76, 0xf7, 0xa1, 0x16, 0x40, 0x2c, 0xbb, 0xb3, 0x62,
            0x43, 0x29, 0x54, 0xed,
        ],
    }];
    encrypted_volume.unlock(&credentials)?;

    let data_stream: DataStreamReference = encrypted_volume.get_data_stream().unwrap();

    let (volume_offset, md5_hash): (u64, String) = read_data_stream(&data_stream)?;

    assert_eq!(volume_offset, encrypted_volume.get_volume_size());
    assert_eq!(md5_hash.as_str(), "3a6a2ee80df6ee2db09faa65cba80726");

    Ok(())
}
