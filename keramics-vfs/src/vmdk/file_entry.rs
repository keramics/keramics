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
use keramics_formats::PathComponent;
use keramics_formats::vmdk::VmdkImage;

use crate::enums::VfsFileType;

/// VMware Virtual Disk (VMDK) storage media image file entry.
pub enum VmdkFileEntry {
    /// Layer file entry.
    Layer {
        /// File.
        image: Arc<RwLock<VmdkImage>>,

        /// Size.
        size: u64,
    },

    /// Root file entry.
    Root {
        /// File.
        image: Arc<RwLock<VmdkImage>>,
    },
}

impl VmdkFileEntry {
    /// Retrieves the default data stream.
    pub fn get_data_stream(&self) -> Result<Option<DataStreamReference>, ErrorTrace> {
        match self {
            VmdkFileEntry::Layer { image, .. } => Ok(Some(image.clone())),
            VmdkFileEntry::Root { .. } => Ok(None),
        }
    }

    /// Retrieves the file type.
    pub fn get_file_type(&self) -> VfsFileType {
        match self {
            VmdkFileEntry::Layer { .. } => VfsFileType::File,
            VmdkFileEntry::Root { .. } => VfsFileType::Directory,
        }
    }

    /// Retrieves the name.
    pub fn get_name(&self) -> PathComponent {
        match self {
            VmdkFileEntry::Layer { .. } => PathComponent::from("vmdk1"),
            VmdkFileEntry::Root { .. } => PathComponent::Root,
        }
    }

    /// Retrieves the size.
    pub fn get_size(&self) -> u64 {
        match self {
            VmdkFileEntry::Layer { size, .. } => *size,
            VmdkFileEntry::Root { .. } => 0,
        }
    }

    /// Retrieves the number of sub file entries.
    pub fn get_number_of_sub_file_entries(&self) -> usize {
        match self {
            VmdkFileEntry::Layer { .. } => 0,
            VmdkFileEntry::Root { .. } => 1,
        }
    }

    /// Retrieves a specific sub file entry.
    pub fn get_sub_file_entry_by_index(
        &self,
        sub_file_entry_index: usize,
    ) -> Result<VmdkFileEntry, ErrorTrace> {
        match self {
            VmdkFileEntry::Layer { .. } => {
                Err(keramics_core::error_trace_new!("No sub file entries"))
            }
            VmdkFileEntry::Root { image } => {
                if sub_file_entry_index != 0 {
                    return Err(keramics_core::error_trace_new!(format!(
                        "No sub file entry with index: {}",
                        sub_file_entry_index
                    )));
                }
                let media_size: u64 = match image.read() {
                    Ok(vmdk_image) => vmdk_image.media_size,
                    Err(error) => {
                        return Err(keramics_core::error_trace_new_with_error!(
                            "Unable to obtain read lock on VMDK image",
                            error
                        ));
                    }
                };
                Ok(VmdkFileEntry::Layer {
                    image: image.clone(),
                    size: media_size,
                })
            }
        }
    }

    /// Determines if the file entry is the root file entry.
    pub fn is_root_file_entry(&self) -> bool {
        match self {
            VmdkFileEntry::Layer { .. } => false,
            VmdkFileEntry::Root { .. } => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_formats::{FileResolverReference, PathComponent, open_os_file_resolver};

    use crate::tests::get_test_data_path;

    fn get_image() -> Result<VmdkImage, ErrorTrace> {
        let mut image: VmdkImage = VmdkImage::new();

        let path_string: String = get_test_data_path("vmdk");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let file_resolver: FileResolverReference = open_os_file_resolver(&path_buf)?;
        let file_name: PathComponent = PathComponent::from("ext2.vmdk");
        image.open(&file_resolver, &file_name)?;

        Ok(image)
    }

    // TODO: add tests for get_data_stream

    #[test]
    fn test_get_file_type() -> Result<(), ErrorTrace> {
        let vmdk_image: VmdkImage = get_image()?;

        let test_image: Arc<RwLock<VmdkImage>> = Arc::new(RwLock::new(vmdk_image));

        let file_entry = VmdkFileEntry::Root {
            image: test_image.clone(),
        };

        let file_type: VfsFileType = file_entry.get_file_type();
        assert_eq!(file_type, VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_get_name() -> Result<(), ErrorTrace> {
        let vmdk_image: VmdkImage = get_image()?;
        let media_size: u64 = vmdk_image.media_size;

        let test_image: Arc<RwLock<VmdkImage>> = Arc::new(RwLock::new(vmdk_image));

        let file_entry = VmdkFileEntry::Root {
            image: test_image.clone(),
        };

        let name: PathComponent = file_entry.get_name();
        assert_eq!(name, PathComponent::Root);

        let file_entry = VmdkFileEntry::Layer {
            image: test_image.clone(),
            size: media_size,
        };

        let name: PathComponent = file_entry.get_name();
        assert_eq!(name, PathComponent::from("vmdk1"));

        Ok(())
    }

    #[test]
    fn test_get_size() -> Result<(), ErrorTrace> {
        let vmdk_image: VmdkImage = get_image()?;
        let media_size: u64 = vmdk_image.media_size;

        let test_image: Arc<RwLock<VmdkImage>> = Arc::new(RwLock::new(vmdk_image));

        let file_entry = VmdkFileEntry::Root {
            image: test_image.clone(),
        };

        let size: u64 = file_entry.get_size();
        assert_eq!(size, 0);

        let file_entry = VmdkFileEntry::Layer {
            image: test_image.clone(),
            size: media_size,
        };

        let size: u64 = file_entry.get_size();
        assert_eq!(size, 4194304);

        Ok(())
    }

    #[test]
    fn test_get_number_of_sub_file_entries() -> Result<(), ErrorTrace> {
        let vmdk_image: VmdkImage = get_image()?;
        let media_size: u64 = vmdk_image.media_size;

        let test_image: Arc<RwLock<VmdkImage>> = Arc::new(RwLock::new(vmdk_image));

        let file_entry = VmdkFileEntry::Root {
            image: test_image.clone(),
        };

        let number_of_sub_file_entries: usize = file_entry.get_number_of_sub_file_entries();
        assert_eq!(number_of_sub_file_entries, 1);

        let file_entry = VmdkFileEntry::Layer {
            image: test_image.clone(),
            size: media_size,
        };

        let number_of_sub_file_entries: usize = file_entry.get_number_of_sub_file_entries();
        assert_eq!(number_of_sub_file_entries, 0);

        Ok(())
    }

    #[test]
    fn test_get_sub_file_entry_by_index() -> Result<(), ErrorTrace> {
        let vmdk_image: VmdkImage = get_image()?;
        let test_image: Arc<RwLock<VmdkImage>> = Arc::new(RwLock::new(vmdk_image));

        let file_entry = VmdkFileEntry::Root {
            image: test_image.clone(),
        };

        let sub_file_entry: VmdkFileEntry = file_entry.get_sub_file_entry_by_index(0)?;

        let name: PathComponent = sub_file_entry.get_name();
        assert_eq!(name, PathComponent::from("vmdk1"));

        let result: Result<VmdkFileEntry, ErrorTrace> = file_entry.get_sub_file_entry_by_index(99);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_is_root_file_entry() -> Result<(), ErrorTrace> {
        let vmdk_image: VmdkImage = get_image()?;
        let media_size: u64 = vmdk_image.media_size;

        let test_image: Arc<RwLock<VmdkImage>> = Arc::new(RwLock::new(vmdk_image));

        let file_entry = VmdkFileEntry::Root {
            image: test_image.clone(),
        };

        assert_eq!(file_entry.is_root_file_entry(), true);

        let file_entry = VmdkFileEntry::Layer {
            image: test_image.clone(),
            size: media_size,
        };

        assert_eq!(file_entry.is_root_file_entry(), false);

        Ok(())
    }
}
