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
use keramics_formats::sparseimage::SparseImageFile;

use crate::enums::VfsFileType;

/// Universal Disk Image Format (UDIF) storage media image file entry.
pub enum SparseImageFileEntry {
    /// Layer file entry.
    Layer {
        /// File.
        file: Arc<RwLock<SparseImageFile>>,
    },

    /// Root file entry.
    Root {
        /// File.
        file: Arc<RwLock<SparseImageFile>>,
    },
}

impl SparseImageFileEntry {
    /// Retrieves the default data stream.
    pub fn get_data_stream(&self) -> io::Result<Option<DataStreamReference>> {
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
    pub fn get_name(&self) -> Option<String> {
        match self {
            SparseImageFileEntry::Layer { .. } => Some("sparseimage1".to_string()),
            SparseImageFileEntry::Root { .. } => None,
        }
    }

    /// Retrieves the number of sub file entries.
    pub fn get_number_of_sub_file_entries(&mut self) -> io::Result<usize> {
        match self {
            SparseImageFileEntry::Layer { .. } => Ok(0),
            SparseImageFileEntry::Root { .. } => Ok(1),
        }
    }

    /// Retrieves a specific sub file entry.
    pub fn get_sub_file_entry_by_index(
        &mut self,
        sub_file_entry_index: usize,
    ) -> io::Result<SparseImageFileEntry> {
        match self {
            SparseImageFileEntry::Layer { .. } => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "No sub file entries",
            )),
            SparseImageFileEntry::Root { file } => {
                if sub_file_entry_index != 0 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("No sub file entry with index: {}", sub_file_entry_index),
                    ));
                }
                Ok(SparseImageFileEntry::Layer { file: file.clone() })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_core::open_os_data_stream;

    fn get_file() -> io::Result<SparseImageFile> {
        let mut file: SparseImageFile = SparseImageFile::new();

        let data_stream: DataStreamReference =
            open_os_data_stream("../test_data/sparseimage/hfsplus.sparseimage")?;
        file.read_data_stream(&data_stream)?;

        Ok(file)
    }

    // TODO: add tests for get_data_stream

    #[test]
    fn test_get_file_type() -> io::Result<()> {
        let sparseimage_file: Arc<RwLock<SparseImageFile>> = Arc::new(RwLock::new(get_file()?));

        let file_entry = SparseImageFileEntry::Root {
            file: sparseimage_file.clone(),
        };

        let file_type: VfsFileType = file_entry.get_file_type();
        assert!(file_type == VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_name() -> io::Result<()> {
        let sparseimage_file: Arc<RwLock<SparseImageFile>> = Arc::new(RwLock::new(get_file()?));

        let file_entry = SparseImageFileEntry::Root {
            file: sparseimage_file.clone(),
        };

        let name: Option<String> = file_entry.get_name();
        assert!(name.is_none());

        let file_entry = SparseImageFileEntry::Layer {
            file: sparseimage_file.clone(),
        };

        let name: Option<String> = file_entry.get_name();
        assert_eq!(name, Some("sparseimage1".to_string()));

        Ok(())
    }

    // TODO: add tests for get_number_of_sub_file_entries
    // TODO: add tests for get_sub_file_entry_by_index
}
