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

use std::sync::{Arc, RwLock};

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_formats::PathComponent;
use keramics_formats::sparseimage::SparseImageFile;

use crate::enums::VfsFileType;

/// Mac OS sparse image (.sparseimage) storage media image file entry.
pub enum SparseImageFileEntry {
    /// Layer file entry.
    Layer {
        /// File.
        file: Arc<RwLock<SparseImageFile>>,

        /// Size.
        size: u64,
    },

    /// Root file entry.
    Root {
        /// File.
        file: Arc<RwLock<SparseImageFile>>,
    },
}

impl SparseImageFileEntry {
    /// Retrieves the default data stream.
    pub fn get_data_stream(&self) -> Result<Option<DataStreamReference>, ErrorTrace> {
        match self {
            SparseImageFileEntry::Layer { file, .. } => Ok(Some(file.clone())),
            SparseImageFileEntry::Root { .. } => Ok(None),
        }
    }

    /// Retrieves the file type.
    pub fn get_file_type(&self) -> VfsFileType {
        match self {
            SparseImageFileEntry::Layer { .. } => VfsFileType::File,
            SparseImageFileEntry::Root { .. } => VfsFileType::Directory,
        }
    }

    /// Retrieves the name.
    pub fn get_name(&self) -> PathComponent {
        match self {
            SparseImageFileEntry::Layer { .. } => PathComponent::from("sparseimage1"),
            SparseImageFileEntry::Root { .. } => PathComponent::Root,
        }
    }

    /// Retrieves the size.
    pub fn get_size(&self) -> u64 {
        match self {
            SparseImageFileEntry::Layer { size, .. } => *size,
            SparseImageFileEntry::Root { .. } => 0,
        }
    }

    /// Retrieves the number of sub file entries.
    pub fn get_number_of_sub_file_entries(&self) -> usize {
        match self {
            SparseImageFileEntry::Layer { .. } => 0,
            SparseImageFileEntry::Root { .. } => 1,
        }
    }

    /// Retrieves a specific sub file entry.
    pub fn get_sub_file_entry_by_index(
        &self,
        sub_file_entry_index: usize,
    ) -> Result<SparseImageFileEntry, ErrorTrace> {
        match self {
            SparseImageFileEntry::Layer { .. } => {
                Err(keramics_core::error_trace_new!("No sub file entries"))
            }
            SparseImageFileEntry::Root { file } => {
                if sub_file_entry_index != 0 {
                    return Err(keramics_core::error_trace_new!(format!(
                        "No sub file entry with index: {}",
                        sub_file_entry_index
                    )));
                }
                let media_size: u64 = match file.read() {
                    Ok(sparseimage_file) => sparseimage_file.media_size,
                    Err(error) => {
                        return Err(keramics_core::error_trace_new_with_error!(
                            "Unable to obtain read lock on sparseimage file",
                            error
                        ));
                    }
                };
                Ok(SparseImageFileEntry::Layer {
                    file: file.clone(),
                    size: media_size,
                })
            }
        }
    }

    /// Determines if the file entry is the root file entry.
    pub fn is_root_file_entry(&self) -> bool {
        match self {
            SparseImageFileEntry::Layer { .. } => false,
            SparseImageFileEntry::Root { .. } => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;

    use crate::tests::get_test_data_path;

    fn get_file() -> Result<SparseImageFile, ErrorTrace> {
        let mut file: SparseImageFile = SparseImageFile::new();

        let path_string: String = get_test_data_path("sparseimage/hfsplus.sparseimage");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file.read_data_stream(&data_stream)?;

        Ok(file)
    }

    // TODO: add tests for get_data_stream

    #[test]
    fn test_get_file_type() -> Result<(), ErrorTrace> {
        let sparseimage_file: SparseImageFile = get_file()?;

        let test_file: Arc<RwLock<SparseImageFile>> = Arc::new(RwLock::new(sparseimage_file));

        let file_entry = SparseImageFileEntry::Root {
            file: test_file.clone(),
        };

        let file_type: VfsFileType = file_entry.get_file_type();
        assert_eq!(file_type, VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_get_name() -> Result<(), ErrorTrace> {
        let sparseimage_file: SparseImageFile = get_file()?;
        let media_size: u64 = sparseimage_file.media_size;

        let test_file: Arc<RwLock<SparseImageFile>> = Arc::new(RwLock::new(sparseimage_file));

        let file_entry = SparseImageFileEntry::Root {
            file: test_file.clone(),
        };

        let name: PathComponent = file_entry.get_name();
        assert_eq!(name, PathComponent::Root);

        let file_entry = SparseImageFileEntry::Layer {
            file: test_file.clone(),
            size: media_size,
        };

        let name: PathComponent = file_entry.get_name();
        assert_eq!(name, PathComponent::from("sparseimage1"));

        Ok(())
    }

    #[test]
    fn test_get_size() -> Result<(), ErrorTrace> {
        let sparseimage_file: SparseImageFile = get_file()?;
        let media_size: u64 = sparseimage_file.media_size;

        let test_file: Arc<RwLock<SparseImageFile>> = Arc::new(RwLock::new(sparseimage_file));

        let file_entry = SparseImageFileEntry::Root {
            file: test_file.clone(),
        };

        let size: u64 = file_entry.get_size();
        assert_eq!(size, 0);

        let file_entry = SparseImageFileEntry::Layer {
            file: test_file.clone(),
            size: media_size,
        };

        let size: u64 = file_entry.get_size();
        assert_eq!(size, 4194304);

        Ok(())
    }

    #[test]
    fn test_get_number_of_sub_file_entries() -> Result<(), ErrorTrace> {
        let sparseimage_file: SparseImageFile = get_file()?;
        let media_size: u64 = sparseimage_file.media_size;

        let test_file: Arc<RwLock<SparseImageFile>> = Arc::new(RwLock::new(sparseimage_file));

        let file_entry = SparseImageFileEntry::Root {
            file: test_file.clone(),
        };

        let number_of_sub_file_entries: usize = file_entry.get_number_of_sub_file_entries();
        assert_eq!(number_of_sub_file_entries, 1);

        let file_entry = SparseImageFileEntry::Layer {
            file: test_file.clone(),
            size: media_size,
        };

        let number_of_sub_file_entries: usize = file_entry.get_number_of_sub_file_entries();
        assert_eq!(number_of_sub_file_entries, 0);

        Ok(())
    }

    #[test]
    fn test_get_sub_file_entry_by_index() -> Result<(), ErrorTrace> {
        let sparseimage_file: SparseImageFile = get_file()?;

        let test_file: Arc<RwLock<SparseImageFile>> = Arc::new(RwLock::new(sparseimage_file));

        let file_entry = SparseImageFileEntry::Root {
            file: test_file.clone(),
        };

        let sub_file_entry: SparseImageFileEntry = file_entry.get_sub_file_entry_by_index(0)?;

        let name: PathComponent = sub_file_entry.get_name();
        assert_eq!(name, PathComponent::from("sparseimage1"));

        let result: Result<SparseImageFileEntry, ErrorTrace> =
            file_entry.get_sub_file_entry_by_index(99);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_is_root_file_entry() -> Result<(), ErrorTrace> {
        let sparseimage_file: SparseImageFile = get_file()?;
        let media_size: u64 = sparseimage_file.media_size;

        let test_file: Arc<RwLock<SparseImageFile>> = Arc::new(RwLock::new(sparseimage_file));

        let file_entry = SparseImageFileEntry::Root {
            file: test_file.clone(),
        };

        assert_eq!(file_entry.is_root_file_entry(), true);

        let file_entry = SparseImageFileEntry::Layer {
            file: test_file.clone(),
            size: media_size,
        };

        assert_eq!(file_entry.is_root_file_entry(), false);

        Ok(())
    }
}
