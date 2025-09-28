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
use std::sync::Arc;

use keramics_core::DataStreamReference;
use keramics_formats::vhdx::{VhdxImage, VhdxImageLayer};

use crate::enums::VfsFileType;

/// Virtual Hard Disk version 2 (VHDX) storage media image file entry.
pub enum VhdxFileEntry {
    /// Layer file entry.
    Layer {
        /// Layer index.
        index: usize,

        /// Layer.
        layer: VhdxImageLayer,
    },

    /// Root file entry.
    Root {
        /// Storage media image.
        image: Arc<VhdxImage>,
    },
}

impl VhdxFileEntry {
    /// Retrieves the default data stream.
    pub fn get_data_stream(&self) -> io::Result<Option<DataStreamReference>> {
        match self {
            VhdxFileEntry::Layer { layer, .. } => Ok(Some(layer.clone())),
            VhdxFileEntry::Root { .. } => Ok(None),
        }
    }

    /// Retrieves the file type.
    pub fn get_file_type(&self) -> VfsFileType {
        match self {
            VhdxFileEntry::Layer { .. } => VfsFileType::File,
            VhdxFileEntry::Root { .. } => VfsFileType::Directory,
        }
    }

    /// Retrieves the name.
    pub fn get_name(&self) -> Option<String> {
        match self {
            VhdxFileEntry::Layer { index, .. } => Some(format!("vhdx{}", index + 1)),
            VhdxFileEntry::Root { .. } => None,
        }
    }

    /// Retrieves the number of sub file entries.
    pub fn get_number_of_sub_file_entries(&mut self) -> io::Result<usize> {
        match self {
            VhdxFileEntry::Layer { .. } => Ok(0),
            VhdxFileEntry::Root { image } => Ok(image.get_number_of_layers()),
        }
    }

    /// Retrieves a specific sub file entry.
    pub fn get_sub_file_entry_by_index(
        &mut self,
        sub_file_entry_index: usize,
    ) -> io::Result<VhdxFileEntry> {
        match self {
            VhdxFileEntry::Layer { .. } => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "No sub file entries",
            )),
            VhdxFileEntry::Root { image } => {
                let vhdx_layer: VhdxImageLayer = image.get_layer_by_index(sub_file_entry_index)?;

                Ok(VhdxFileEntry::Layer {
                    index: sub_file_entry_index,
                    layer: vhdx_layer.clone(),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_core::{FileResolverReference, open_os_file_resolver};

    fn get_image() -> io::Result<VhdxImage> {
        let mut image: VhdxImage = VhdxImage::new();

        let file_resolver: FileResolverReference = open_os_file_resolver("../test_data/vhdx")?;
        image.open(&file_resolver, "ntfs-differential.vhdx")?;

        Ok(image)
    }

    // TODO: add tests for get_data_stream

    #[test]
    fn test_get_file_type() -> io::Result<()> {
        let vhdx_image: Arc<VhdxImage> = Arc::new(get_image()?);

        let file_entry = VhdxFileEntry::Root {
            image: vhdx_image.clone(),
        };

        let file_type: VfsFileType = file_entry.get_file_type();
        assert!(file_type == VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_name() -> io::Result<()> {
        let vhdx_image: Arc<VhdxImage> = Arc::new(get_image()?);

        let file_entry = VhdxFileEntry::Root {
            image: vhdx_image.clone(),
        };

        let name: Option<String> = file_entry.get_name();
        assert!(name.is_none());

        let vhdx_layer: VhdxImageLayer = vhdx_image.get_layer_by_index(0)?;

        let file_entry = VhdxFileEntry::Layer {
            index: 0,
            layer: vhdx_layer.clone(),
        };

        let name: Option<String> = file_entry.get_name();
        assert_eq!(name, Some("vhdx1".to_string()));

        Ok(())
    }

    // TODO: add tests for get_number_of_sub_file_entries
    // TODO: add tests for get_sub_file_entry_by_index
}
