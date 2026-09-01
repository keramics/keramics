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

use std::io::SeekFrom;
use std::sync::{Arc, RwLock};

use keramics_core::{DataStreamReference, ErrorTrace};

use crate::cdsaencr::constants::*;
use crate::cdsaencr::{
    CdsaEncrContainer, CdsaEncrCredential, CdsaEncrEncryptionContext, CdsaEncrEncryptionType,
};
use crate::fake_file_resolver::FakeFileResolver;
use crate::file_resolver::FileResolverReference;
use crate::path_component::PathComponent;
use crate::plist::XmlPlist;

use super::block_reader::SparseBundleBlockReader;
use super::block_stream::SparseBundleBlockStream;

/// Mac OS sparse bundle (.sparsebundle) storage media image.
pub struct SparseBundleImage {
    /// File resolver.
    file_resolver: FileResolverReference,

    /// Bytes per sector.
    bytes_per_sector: u16,

    /// Band size.
    band_size: u32,

    /// Encrypted container.
    encrypted_container: Option<CdsaEncrContainer>,

    /// Encrypted block size.
    encrypted_block_size: usize,

    /// Media size.
    media_size: u64,
}

impl SparseBundleImage {
    /// Creates a new storage media image.
    pub fn new() -> Self {
        Self {
            file_resolver: FileResolverReference::new(Box::new(FakeFileResolver::new())),
            bytes_per_sector: 0,
            band_size: 0,
            encrypted_container: None,
            encrypted_block_size: 0,
            media_size: 0,
        }
    }

    /// Retrieves the block size.
    pub fn get_block_size(&self) -> u32 {
        self.band_size
    }

    /// Retrieves the bytes per sector.
    pub fn get_bytes_per_sector(&self) -> u16 {
        self.bytes_per_sector
    }

    /// Retrieves a data stream.
    pub fn get_data_stream(&self) -> DataStreamReference {
        let encryption_context: Option<&CdsaEncrEncryptionContext> = match &self.encrypted_container
        {
            Some(encrypted_container) => encrypted_container.encryption_context.as_ref(),
            None => None,
        };
        Arc::new(RwLock::new(SparseBundleBlockStream::new(
            SparseBundleBlockReader::new(
                &self.file_resolver,
                self.band_size,
                encryption_context,
                self.encrypted_block_size,
                self.media_size,
            ),
        )))
    }

    /// Retrieves the encryption type.
    pub fn get_encryption_type(&self) -> Option<&CdsaEncrEncryptionType> {
        match &self.encrypted_container {
            Some(encrypted_container) => Some(encrypted_container.get_encryption_type()),
            None => None,
        }
    }

    /// Retrieves the media size.
    pub fn get_media_size(&self) -> u64 {
        self.media_size
    }

    /// Determines if the (encrypted) image is locked.
    pub fn is_locked(&self) -> bool {
        match &self.encrypted_container {
            Some(encrypted_container) => encrypted_container.is_locked(),
            None => false,
        }
    }

    /// Opens a storage media image.
    pub fn open(
        &mut self,
        file_resolver: &FileResolverReference,
        file_name: &PathComponent,
    ) -> Result<(), ErrorTrace> {
        match self.read_info_plist_file(&file_resolver, file_name) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!("Unable to read info.plist file: {}", file_name)
                );
                return Err(error);
            }
        }
        match self.read_token_file(&file_resolver) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read token file");
                return Err(error);
            }
        }
        self.file_resolver = file_resolver.clone();

        Ok(())
    }

    /// Reads an Info.plist or Info.bckup file.
    fn read_info_plist_file(
        &mut self,
        file_resolver: &FileResolverReference,
        file_name: &PathComponent,
    ) -> Result<(), ErrorTrace> {
        let path_components: [PathComponent; 1] = [file_name.clone()];

        let data_stream: DataStreamReference = match file_resolver.get_data_stream(&path_components)
        {
            Ok(Some(data_stream)) => data_stream,
            Ok(None) => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Missing data stream: {}",
                    file_name
                )));
            }
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!("Unable to open file: {}", file_name)
                );
                return Err(error);
            }
        };
        let data_stream_size: u64 = keramics_core::data_stream_get_size!(data_stream);

        if data_stream_size == 0 || data_stream_size > 65536 {
            return Err(keramics_core::error_trace_new!("Unsupported file size"));
        }
        let mut data: Vec<u8> = vec![0; data_stream_size as usize];

        keramics_core::data_stream_read_at_position!(data_stream, &mut data, SeekFrom::Start(0));

        keramics_core::debug_trace_data!("SparseBundleImageXmlPlist", 0, &data, data_stream_size);

        let string: String = match String::from_utf8(data) {
            Ok(string) => string,
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to convert XML plist data into UTF-8 string",
                    error
                ));
            }
        };
        let mut xml_plist: XmlPlist = XmlPlist::new();

        match xml_plist.parse(string.as_str()) {
            Ok(_) => {}
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to parse XML plist",
                    error
                ));
            }
        }
        match xml_plist
            .root_object
            .get_string_by_key("CFBundleInfoDictionaryVersion")
        {
            Some(string) => {
                if string != "6.0" {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported CFBundleInfoDictionaryVersion: {}",
                        string
                    )));
                }
            }
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Unable to retrieve CFBundleInfoDictionaryVersion value"
                ));
            }
        }
        match xml_plist
            .root_object
            .get_string_by_key("diskimage-bundle-type")
        {
            Some(string) => {
                if string != "com.apple.diskimage.sparsebundle" {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported diskimage-bundle-type: {}",
                        string
                    )));
                }
            }
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Unable to retrieve diskimage-bundle-type value"
                ));
            }
        }
        match xml_plist.root_object.get_integer_by_key("band-size") {
            Some(integer) => {
                if *integer == 0 || *integer > u32::MAX as i64 {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Invalid band-size: {} value out of bounds",
                        *integer
                    )));
                }
                self.band_size = *integer as u32;
            }
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Unable to retrieve band-size value"
                ));
            }
        }
        match xml_plist.root_object.get_integer_by_key("size") {
            Some(integer) => {
                if *integer == 0 {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Invalid size: {} value out of bounds",
                        *integer
                    )));
                }
                self.media_size = *integer as u64;
            }
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Unable to retrieve size value"
                ));
            }
        }
        self.bytes_per_sector = 512;

        Ok(())
    }

    /// Reads a token file.
    fn read_token_file(&mut self, file_resolver: &FileResolverReference) -> Result<(), ErrorTrace> {
        let path_components: [PathComponent; 1] = [PathComponent::from("token")];

        let data_stream: DataStreamReference = match file_resolver.get_data_stream(&path_components)
        {
            Ok(Some(data_stream)) => data_stream,
            Ok(None) => {
                return Err(keramics_core::error_trace_new!(
                    "Missing data stream: token",
                ));
            }
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open file: token");
                return Err(error);
            }
        };
        let data_stream_size: u64 = keramics_core::data_stream_get_size!(data_stream);

        if data_stream_size == 0 {
            return Ok(());
        }
        let mut footer_signature: [u8; 8] = [0; 8];
        let mut header_signature: [u8; 8] = [0; 8];

        keramics_core::data_stream_read_exact_at_position!(
            &data_stream,
            &mut header_signature,
            SeekFrom::Start(0)
        );
        keramics_core::data_stream_read_exact_at_position!(
            &data_stream,
            &mut footer_signature,
            SeekFrom::End(-8)
        );
        if &header_signature == CDSAENCR_CONTAINER_HEADER_SIGNATURE
            || &footer_signature == CDSAENCR_CONTAINER_FOOTER_SIGNATURE
        {
            let mut encrypted_container: CdsaEncrContainer = CdsaEncrContainer::new();

            match encrypted_container.read_data_stream(&data_stream) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to read encrypted container"
                    );
                    return Err(error);
                }
            }
            self.encrypted_block_size = encrypted_container.get_block_size() as usize;
            self.encrypted_container = Some(encrypted_container);
        }
        Ok(())
    }

    /// Unlocks a locked (encrypted) image.
    pub fn unlock(&mut self, credentials: &[CdsaEncrCredential]) -> Result<bool, ErrorTrace> {
        match &mut self.encrypted_container {
            Some(encrypted_container) => match encrypted_container.unlock(credentials) {
                Ok(result) => Ok(result),
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to unlock encrypted container"
                    );
                    Err(error)
                }
            },
            None => Ok(true),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use crate::os_file_resolver::open_os_file_resolver;

    use crate::tests::get_test_data_path;

    fn get_image(path_string: &str) -> Result<SparseBundleImage, ErrorTrace> {
        let mut image: SparseBundleImage = SparseBundleImage::new();

        let test_path_string: String = get_test_data_path(path_string);
        let path_buf: PathBuf = PathBuf::from(test_path_string.as_str());
        let file_resolver: FileResolverReference = open_os_file_resolver(&path_buf)?;
        let file_name: PathComponent = PathComponent::from("Info.plist");
        image.open(&file_resolver, &file_name)?;

        Ok(image)
    }

    #[test]
    fn test_get_block_size() -> Result<(), ErrorTrace> {
        let image: SparseBundleImage = get_image("sparsebundle/hfsplus.sparsebundle")?;

        let block_size: u32 = image.get_block_size();
        assert_eq!(block_size, 8388608);

        Ok(())
    }

    #[test]
    fn test_get_bytes_per_sector() -> Result<(), ErrorTrace> {
        let image: SparseBundleImage = get_image("sparsebundle/hfsplus.sparsebundle")?;

        let bytes_per_sector: u16 = image.get_bytes_per_sector();
        assert_eq!(bytes_per_sector, 512);

        Ok(())
    }

    #[test]
    fn test_get_encryption_type() -> Result<(), ErrorTrace> {
        let image: SparseBundleImage = get_image("sparsebundle/hfsplus_aes128.sparsebundle")?;

        let encryption_type: &CdsaEncrEncryptionType = image.get_encryption_type().unwrap();
        assert_eq!(encryption_type.method, 0x80000001);
        assert_eq!(encryption_type.mode, 5);
        assert_eq!(encryption_type.key_size, 16);

        Ok(())
    }

    #[test]
    fn test_get_media_size() -> Result<(), ErrorTrace> {
        let image: SparseBundleImage = get_image("sparsebundle/hfsplus.sparsebundle")?;

        let media_size: u64 = image.get_media_size();
        assert_eq!(media_size, 4194304);

        Ok(())
    }

    #[test]
    fn test_is_locked() -> Result<(), ErrorTrace> {
        let image: SparseBundleImage = get_image("sparsebundle/hfsplus_aes128.sparsebundle")?;

        let is_locked: bool = image.is_locked();
        assert_eq!(is_locked, true);

        Ok(())
    }

    #[test]
    fn test_open() -> Result<(), ErrorTrace> {
        let mut image: SparseBundleImage = SparseBundleImage::new();

        let path_string: String = get_test_data_path("sparsebundle/hfsplus.sparsebundle");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let file_resolver: FileResolverReference = open_os_file_resolver(&path_buf)?;
        let file_name: PathComponent = PathComponent::from("Info.plist");
        image.open(&file_resolver, &file_name)?;

        assert_eq!(image.bytes_per_sector, 512);
        assert_eq!(image.band_size, 8388608);
        assert_eq!(image.media_size, 4194304);

        Ok(())
    }

    #[test]
    fn test_read_info_plist_file() -> Result<(), ErrorTrace> {
        let mut image: SparseBundleImage = SparseBundleImage::new();

        let path_string: String = get_test_data_path("sparsebundle/hfsplus.sparsebundle");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let file_resolver: FileResolverReference = open_os_file_resolver(&path_buf)?;
        let file_name: PathComponent = PathComponent::from("Info.plist");
        image.read_info_plist_file(&file_resolver, &file_name)?;

        assert_eq!(image.bytes_per_sector, 512);
        assert_eq!(image.band_size, 8388608);
        assert_eq!(image.media_size, 4194304);

        Ok(())
    }

    #[test]
    fn test_unlock() -> Result<(), ErrorTrace> {
        let mut image: SparseBundleImage = get_image("sparsebundle/hfsplus_aes128.sparsebundle")?;

        assert_eq!(image.is_locked(), true);

        let credentials: Vec<CdsaEncrCredential> =
            vec![CdsaEncrCredential::Passphrase(b"KeRaMiCs".to_vec())];
        image.unlock(&credentials)?;

        assert_eq!(image.is_locked(), false);

        Ok(())
    }
}
