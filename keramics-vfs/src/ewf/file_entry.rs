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

use std::sync::{Arc, RwLock};

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_formats::ewf::EwfImage;

use crate::enums::VfsFileType;

/// Expert Witness Compression Format (EWF) storage media image file entry.
pub enum EwfFileEntry {
    /// Layer file entry.
    Layer {
        /// File.
        image: Arc<RwLock<EwfImage>>,

        /// Size.
        size: u64,
    },

    /// Root file entry.
    Root {
        /// File.
        image: Arc<RwLock<EwfImage>>,
    },
}

impl EwfFileEntry {
    /// Retrieves the default data stream.
    pub fn get_data_stream(&self) -> Result<Option<DataStreamReference>, ErrorTrace> {
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
            EwfFileEntry::Layer { .. } => Some(String::from("ewf1")),
            EwfFileEntry::Root { .. } => None,
        }
    }

    /// Retrieves the size.
    pub fn get_size(&self) -> u64 {
        match self {
            EwfFileEntry::Layer { size, .. } => *size,
            EwfFileEntry::Root { .. } => 0,
        }
    }

    /// Retrieves the number of sub file entries.
    pub fn get_number_of_sub_file_entries(&self) -> Result<usize, ErrorTrace> {
        match self {
            EwfFileEntry::Layer { .. } => Ok(0),
            EwfFileEntry::Root { .. } => Ok(1),
        }
    }

    /// Retrieves a specific sub file entry.
    pub fn get_sub_file_entry_by_index(
        &self,
        sub_file_entry_index: usize,
    ) -> Result<EwfFileEntry, ErrorTrace> {
        match self {
            EwfFileEntry::Layer { .. } => {
                Err(keramics_core::error_trace_new!("No sub file entries"))
            }
            EwfFileEntry::Root { image } => {
                if sub_file_entry_index != 0 {
                    return Err(keramics_core::error_trace_new!(format!(
                        "No sub file entry with index: {}",
                        sub_file_entry_index
                    )));
                }
                let media_size: u64 = match image.read() {
                    Ok(ewf_image) => ewf_image.media_size,
                    Err(error) => {
                        return Err(keramics_core::error_trace_new_with_error!(
                            "Unable to obtain read lock on EWF image",
                            error
                        ));
                    }
                };
                Ok(EwfFileEntry::Layer {
                    image: image.clone(),
                    size: media_size,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_formats::{FileResolverReference, PathComponent, open_os_file_resolver};

    use crate::tests::get_test_data_path;

    fn get_image() -> Result<EwfImage, ErrorTrace> {
        let mut image: EwfImage = EwfImage::new();

        let path_buf: PathBuf = PathBuf::from(get_test_data_path("ewf").as_str());
        let file_resolver: FileResolverReference = open_os_file_resolver(&path_buf)?;
        let file_name: PathComponent = PathComponent::from("ext2.E01");
        image.open(&file_resolver, &file_name)?;

        Ok(image)
    }

    // TODO: add tests for get_data_stream

    #[test]
    fn test_get_file_type() -> Result<(), ErrorTrace> {
        let ewf_image: EwfImage = get_image()?;

        let test_image: Arc<RwLock<EwfImage>> = Arc::new(RwLock::new(ewf_image));

        let file_entry = EwfFileEntry::Root {
            image: test_image.clone(),
        };

        let file_type: VfsFileType = file_entry.get_file_type();
        assert!(file_type == VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_get_name() -> Result<(), ErrorTrace> {
        let ewf_image: EwfImage = get_image()?;
        let media_size: u64 = ewf_image.media_size;

        let test_image: Arc<RwLock<EwfImage>> = Arc::new(RwLock::new(ewf_image));

        let file_entry = EwfFileEntry::Root {
            image: test_image.clone(),
        };

        let name: Option<String> = file_entry.get_name();
        assert!(name.is_none());

        let file_entry = EwfFileEntry::Layer {
            image: test_image.clone(),
            size: media_size,
        };

        let name: Option<String> = file_entry.get_name();
        assert_eq!(name, Some(String::from("ewf1")));

        Ok(())
    }

    #[test]
    fn test_get_size() -> Result<(), ErrorTrace> {
        let ewf_image: EwfImage = get_image()?;
        let media_size: u64 = ewf_image.media_size;

        let test_image: Arc<RwLock<EwfImage>> = Arc::new(RwLock::new(ewf_image));

        let file_entry = EwfFileEntry::Root {
            image: test_image.clone(),
        };

        let size: u64 = file_entry.get_size();
        assert_eq!(size, 0);

        let file_entry = EwfFileEntry::Layer {
            image: test_image.clone(),
            size: media_size,
        };

        let size: u64 = file_entry.get_size();
        assert_eq!(size, 4194304);

        Ok(())
    }

    #[test]
    fn test_get_number_of_sub_file_entries() -> Result<(), ErrorTrace> {
        let ewf_image: EwfImage = get_image()?;
        let media_size: u64 = ewf_image.media_size;

        let test_image: Arc<RwLock<EwfImage>> = Arc::new(RwLock::new(ewf_image));

        let file_entry = EwfFileEntry::Root {
            image: test_image.clone(),
        };

        let number_of_sub_file_entries: usize = file_entry.get_number_of_sub_file_entries()?;
        assert_eq!(number_of_sub_file_entries, 1);

        let file_entry = EwfFileEntry::Layer {
            image: test_image.clone(),
            size: media_size,
        };

        let number_of_sub_file_entries: usize = file_entry.get_number_of_sub_file_entries()?;
        assert_eq!(number_of_sub_file_entries, 0);

        Ok(())
    }

    #[test]
    fn test_get_sub_file_entry_by_index() -> Result<(), ErrorTrace> {
        let ewf_image: EwfImage = get_image()?;
        let test_image: Arc<RwLock<EwfImage>> = Arc::new(RwLock::new(ewf_image));

        let file_entry = EwfFileEntry::Root {
            image: test_image.clone(),
        };

        let sub_file_entry: EwfFileEntry = file_entry.get_sub_file_entry_by_index(0)?;
        assert_eq!(sub_file_entry.get_name(), Some(String::from("ewf1")));

        let result: Result<EwfFileEntry, ErrorTrace> = file_entry.get_sub_file_entry_by_index(99);
        assert!(result.is_err());

        Ok(())
    }
}
