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

use std::io;
use std::sync::{Arc, RwLock};

use keramics_core::DataStreamReference;
use keramics_formats::ewf::EwfImage;

use crate::enums::VfsFileType;

/// Expert Witness Compression Format (EWF) storage media image file entry.
pub enum EwfFileEntry {
    /// Layer file entry.
    Layer {
        /// File.
        image: Arc<RwLock<EwfImage>>,
    },

    /// Root file entry.
    Root {
        /// File.
        image: Arc<RwLock<EwfImage>>,
    },
}

impl EwfFileEntry {
    /// Retrieves the default data stream.
    pub fn get_data_stream(&self) -> io::Result<Option<DataStreamReference>> {
        match self {
            EwfFileEntry::Layer { image, .. } => Ok(Some(image.clone())),
            EwfFileEntry::Root { .. } => Ok(None),
        }
    }

    /// Retrieves the file type.
    pub fn get_file_type(&self) -> VfsFileType {
        match self {
            EwfFileEntry::Layer { .. } => VfsFileType::File,
            EwfFileEntry::Root { .. } => VfsFileType::Directory,
        }
    }

    /// Retrieves the name.
    pub fn get_name(&self) -> Option<String> {
        match self {
            EwfFileEntry::Layer { .. } => Some("ewf1".to_string()),
            EwfFileEntry::Root { .. } => None,
        }
    }

    /// Retrieves the number of sub file entries.
    pub fn get_number_of_sub_file_entries(&mut self) -> io::Result<usize> {
        match self {
            EwfFileEntry::Layer { .. } => Ok(0),
            EwfFileEntry::Root { .. } => Ok(1),
        }
    }

    /// Retrieves a specific sub file entry.
    pub fn get_sub_file_entry_by_index(
        &mut self,
        sub_file_entry_index: usize,
    ) -> io::Result<EwfFileEntry> {
        match self {
            EwfFileEntry::Layer { .. } => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "No sub file entries",
            )),
            EwfFileEntry::Root { image } => {
                if sub_file_entry_index != 0 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("No sub file entry with index: {}", sub_file_entry_index),
                    ));
                }
                Ok(EwfFileEntry::Layer {
                    image: image.clone(),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_core::{FileResolverReference, open_os_file_resolver};

    fn get_image() -> io::Result<EwfImage> {
        let mut image: EwfImage = EwfImage::new();

        let file_resolver: FileResolverReference = open_os_file_resolver("../test_data/ewf")?;
        image.open(&file_resolver, "ext2.E01")?;

        Ok(image)
    }

    // TODO: add tests for get_data_stream

    #[test]
    fn test_get_file_type() -> io::Result<()> {
        let ewf_image: Arc<RwLock<EwfImage>> = Arc::new(RwLock::new(get_image()?));

        let file_entry = EwfFileEntry::Root {
            image: ewf_image.clone(),
        };

        let file_type: VfsFileType = file_entry.get_file_type();
        assert!(file_type == VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_name() -> io::Result<()> {
        let ewf_image: Arc<RwLock<EwfImage>> = Arc::new(RwLock::new(get_image()?));

        let file_entry = EwfFileEntry::Root {
            image: ewf_image.clone(),
        };

        let name: Option<String> = file_entry.get_name();
        assert!(name.is_none());

        let file_entry = EwfFileEntry::Layer {
            image: ewf_image.clone(),
        };

        let name: Option<String> = file_entry.get_name();
        assert_eq!(name, Some("ewf1".to_string()));

        Ok(())
    }

    // TODO: add tests for get_number_of_sub_file_entries
    // TODO: add tests for get_sub_file_entry_by_index
}
