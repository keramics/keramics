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

use std::sync::Arc;

use keramics_core::{DataStreamReference, ErrorTrace};

use crate::path::Path;

use super::constants::*;
use super::file_entry::ApfsFileEntry;
use super::file_system_tree::ApfsFileSystemTree;
use super::inode::ApfsInode;
use super::object_map_tree::ApfsObjectMapTree;

/// Apple File System (APFS) file system.
pub struct ApfsFileSystem {
    /// The data stream.
    data_stream: Option<DataStreamReference>,

    /// Block size.
    block_size: u32,

    /// Object map B-tree.
    object_map_tree: Arc<ApfsObjectMapTree>,

    /// File system B-tree.
    file_system_tree: Arc<ApfsFileSystemTree>,

    /// Transaction identifier.
    transaction_identifier: u64,
    // TODO: add encryption context.
}

impl ApfsFileSystem {
    /// Creates a file system.
    pub(super) fn new(
        block_size: u32,
        object_map_tree: &Arc<ApfsObjectMapTree>,
        use_case_folding: bool,
    ) -> Self {
        Self {
            data_stream: None,
            block_size,
            object_map_tree: object_map_tree.clone(),
            file_system_tree: Arc::new(ApfsFileSystemTree::new(use_case_folding)),
            transaction_identifier: 0,
        }
    }

    /// Retrieves the file entry for a specific identifier.
    pub fn get_file_entry_by_identifier(
        &self,
        identifier: u64,
    ) -> Result<Option<ApfsFileEntry>, ErrorTrace> {
        let data_stream: &DataStreamReference = match self.data_stream.as_ref() {
            Some(data_stream) => data_stream,
            None => {
                return Err(keramics_core::error_trace_new!("Missing data stream"));
            }
        };
        let inode: ApfsInode = match self.file_system_tree.get_inode_by_identifier(
            data_stream,
            &self.object_map_tree,
            identifier,
            self.transaction_identifier,
        ) {
            Ok(Some(inode)) => inode,
            Ok(None) => return Ok(None),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!("Unable to retrieve inode: {}", identifier)
                );
                return Err(error);
            }
        };
        let mut file_entry: ApfsFileEntry = ApfsFileEntry::new(
            data_stream,
            self.block_size,
            &self.object_map_tree,
            &self.file_system_tree,
            identifier,
            self.transaction_identifier,
            inode,
            None,
        );
        match file_entry.read_attributes() {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read attributes");
                return Err(error);
            }
        }
        match file_entry.read_extents() {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read extents");
                return Err(error);
            }
        }
        Ok(Some(file_entry))
    }

    /// Retrieves the file entry for a specific path.
    pub fn get_file_entry_by_path(&self, path: &Path) -> Result<Option<ApfsFileEntry>, ErrorTrace> {
        if path.is_empty() || path.is_relative() {
            return Ok(None);
        }
        let mut file_entry: ApfsFileEntry = match self.get_root_directory() {
            Ok(Some(file_entry)) => file_entry,
            Ok(None) => return Ok(None),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to retrieve root directory");
                return Err(error);
            }
        };
        // TODO: cache file entries.
        for path_component in path.components[1..].iter() {
            file_entry = match file_entry.get_sub_file_entry_by_name(path_component) {
                Ok(Some(file_entry)) => file_entry,
                Ok(None) => return Ok(None),
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to retrieve sub file entry: {}", path_component)
                    );
                    return Err(error);
                }
            };
        }
        Ok(Some(file_entry))
    }

    /// Retrieves the root directory (file entry).
    pub fn get_root_directory(&self) -> Result<Option<ApfsFileEntry>, ErrorTrace> {
        match self.get_file_entry_by_identifier(APFS_ROOT_DIRECTORY_IDENTIFIER) {
            Ok(result) => Ok(result),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to retrieve file entry: {}",
                        APFS_ROOT_DIRECTORY_IDENTIFIER
                    )
                );
                Err(error)
            }
        }
    }

    /// Opens a file system.
    pub(super) fn open(
        &mut self,
        data_stream: &DataStreamReference,
        root_block_number: u64,
        transaction_identifier: u64,
    ) -> Result<(), ErrorTrace> {
        match Arc::get_mut(&mut self.file_system_tree) {
            Some(file_system_tree) => {
                file_system_tree.initialize(self.block_size, root_block_number);
            }
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Unable to obtain mutable reference to file sytem tree"
                ));
            }
        }
        self.transaction_identifier = transaction_identifier;
        self.data_stream = Some(data_stream.clone());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;

    use crate::apfs::{ApfsContainer, ApfsVolume};

    use crate::tests::get_test_data_path;

    fn get_file_system() -> Result<ApfsFileSystem, ErrorTrace> {
        let mut container: ApfsContainer = ApfsContainer::new();

        let path_string: String = get_test_data_path("apfs/apfs.raw");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        container.read_data_stream(&data_stream)?;

        let volume: ApfsVolume = container.get_volume_by_index(0)?;
        volume.get_file_system()
    }

    #[test]
    fn test_get_file_entry_by_identifier() -> Result<(), ErrorTrace> {
        let file_system: ApfsFileSystem = get_file_system()?;

        let file_entry: ApfsFileEntry = file_system.get_file_entry_by_identifier(19)?.unwrap();
        assert_eq!(file_entry.identifier, 19);

        let result: Option<ApfsFileEntry> = file_system.get_file_entry_by_identifier(0xffffffff)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path() -> Result<(), ErrorTrace> {
        let file_system: ApfsFileSystem = get_file_system()?;

        let path: Path = Path::from("/testdir1/TestFile2");
        let file_entry: ApfsFileEntry = file_system.get_file_entry_by_path(&path)?.unwrap();
        assert_eq!(file_entry.identifier, 19);

        let path: Path = Path::from("/bogus");
        let result: Option<ApfsFileEntry> = file_system.get_file_entry_by_path(&path)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_root_directory() -> Result<(), ErrorTrace> {
        let file_system: ApfsFileSystem = get_file_system()?;

        let file_entry: ApfsFileEntry = file_system.get_root_directory()?.unwrap();

        assert_eq!(file_entry.identifier, 2);

        Ok(())
    }
}
