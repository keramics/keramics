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

use std::sync::Arc;

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_formats::vhd::{VhdImage, VhdImageLayer};

use crate::enums::VfsFileType;

/// QEMU Copy-On-Write (QCOW) storage media image file entry.
pub enum VhdFileEntry {
    /// Layer file entry.
    Layer {
        /// Layer index.
        index: usize,

        /// Layer.
        layer: VhdImageLayer,
    },

    /// Root file entry.
    Root {
        /// Storage media image.
        image: Arc<VhdImage>,
    },
}

impl VhdFileEntry {
    /// Retrieves the default data stream.
    pub fn get_data_stream(&self) -> Result<Option<DataStreamReference>, ErrorTrace> {
        match self {
            VhdFileEntry::Layer { layer, .. } => Ok(Some(layer.clone())),
            VhdFileEntry::Root { .. } => Ok(None),
        }
    }

    /// Retrieves the file type.
    pub fn get_file_type(&self) -> VfsFileType {
        match self {
            VhdFileEntry::Layer { .. } => VfsFileType::File,
            VhdFileEntry::Root { .. } => VfsFileType::Directory,
        }
    }

    /// Retrieves the name.
    pub fn get_name(&self) -> Option<String> {
        match self {
            VhdFileEntry::Layer { index, .. } => Some(format!("vhd{}", index + 1)),
            VhdFileEntry::Root { .. } => None,
        }
    }

    /// Retrieves the number of sub file entries.
    pub fn get_number_of_sub_file_entries(&mut self) -> Result<usize, ErrorTrace> {
        match self {
            VhdFileEntry::Layer { .. } => Ok(0),
            VhdFileEntry::Root { image } => Ok(image.get_number_of_layers()),
        }
    }

    /// Retrieves a specific sub file entry.
    pub fn get_sub_file_entry_by_index(
        &mut self,
        sub_file_entry_index: usize,
    ) -> Result<VhdFileEntry, ErrorTrace> {
        match self {
            VhdFileEntry::Layer { .. } => {
                Err(keramics_core::error_trace_new!("No sub file entries"))
            }
            VhdFileEntry::Root { image } => match image.get_layer_by_index(sub_file_entry_index) {
                Ok(vhd_layer) => Ok(VhdFileEntry::Layer {
                    index: sub_file_entry_index,
                    layer: vhd_layer.clone(),
                }),
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to retrieve VHD image layer: {}",
                            sub_file_entry_index
                        )
                    );
                    return Err(error);
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_core::{FileResolverReference, open_os_file_resolver};

    fn get_image() -> Result<VhdImage, ErrorTrace> {
        let mut image: VhdImage = VhdImage::new();

        let file_resolver: FileResolverReference = open_os_file_resolver("../test_data/vhd")?;
        image.open(&file_resolver, "ntfs-differential.vhd")?;

        Ok(image)
    }

    // TODO: add tests for get_data_stream

    #[test]
    fn test_get_file_type() -> Result<(), ErrorTrace> {
        let vhd_image: Arc<VhdImage> = Arc::new(get_image()?);

        let file_entry = VhdFileEntry::Root {
            image: vhd_image.clone(),
        };

        let file_type: VfsFileType = file_entry.get_file_type();
        assert!(file_type == VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_name() -> Result<(), ErrorTrace> {
        let vhd_image: Arc<VhdImage> = Arc::new(get_image()?);

        let file_entry = VhdFileEntry::Root {
            image: vhd_image.clone(),
        };

        let name: Option<String> = file_entry.get_name();
        assert!(name.is_none());

        let vhd_layer: VhdImageLayer = vhd_image.get_layer_by_index(0)?;
        let file_entry = VhdFileEntry::Layer {
            index: 0,
            layer: vhd_layer.clone(),
        };

        let name: Option<String> = file_entry.get_name();
        assert_eq!(name, Some("vhd1".to_string()));

        Ok(())
    }

    // TODO: add tests for get_number_of_sub_file_entries
    // TODO: add tests for get_sub_file_entry_by_index
}
