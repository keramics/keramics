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
use keramics_formats::PathComponent;

use crate::enums::VfsFileType;
use crate::traits::{VfsImage, VfsImageLayer};

use super::identifier::VfsImageIdentifier;

/// Virtual File System (VFS) image based file entry.
pub enum VfsImageFileEntry<I: VfsImage<Layer = L>, L: VfsImageLayer> {
    /// Layer file entry.
    Layer {
        /// File name index.
        name_index: usize,

        /// Image layer.
        layer: Arc<L>,
    },

    /// Root file entry.
    Root {
        /// Storage media image.
        image: Arc<I>,
    },
}

impl<I: VfsImage<Layer = L>, L: VfsImageLayer> VfsImageFileEntry<I, L> {
    /// Retrieves the default data stream.
    pub fn get_data_stream(&self) -> Result<Option<DataStreamReference>, ErrorTrace> {
        Ok(match self {
            VfsImageFileEntry::Layer { layer, .. } => layer.get_data_stream(),
            VfsImageFileEntry::Root { .. } => None,
        })
    }

    /// Retrieves the file type.
    pub fn get_file_type(&self) -> VfsFileType {
        match self {
            VfsImageFileEntry::Layer { .. } => VfsFileType::File,
            VfsImageFileEntry::Root { .. } => VfsFileType::Directory,
        }
    }

    /// Retrieves the identifier.
    pub fn get_identifier(&self) -> Option<VfsImageIdentifier> {
        match self {
            VfsImageFileEntry::Layer { layer, .. } => layer.get_identifier(),
            VfsImageFileEntry::Root { .. } => None,
        }
    }

    /// Retrieves the (image) layer number.
    pub fn get_layer_number(&self) -> Option<usize> {
        match self {
            VfsImageFileEntry::Layer { name_index, .. } => Some(name_index + 1),
            VfsImageFileEntry::Root { .. } => None,
        }
    }

    /// Retrieves the name.
    pub fn get_name(&self) -> PathComponent {
        match self {
            VfsImageFileEntry::Layer { name_index, .. } => {
                PathComponent::from(format!("{}{}", L::NAME_PREFIX, name_index + 1))
            }
            VfsImageFileEntry::Root { .. } => PathComponent::Root,
        }
    }

    /// Retrieves the size.
    pub fn get_size(&self) -> u64 {
        match self {
            VfsImageFileEntry::Layer { layer, .. } => layer.get_media_size(),
            VfsImageFileEntry::Root { .. } => 0,
        }
    }
    /// Retrieves the number of sub file entries.
    pub fn get_number_of_sub_file_entries(&self) -> usize {
        match self {
            VfsImageFileEntry::Layer { .. } => 0,
            VfsImageFileEntry::Root { image } => image.get_number_of_layers(),
        }
    }

    /// Retrieves a specific sub file entry.
    pub fn get_sub_file_entry_by_index(
        &self,
        sub_file_entry_index: usize,
    ) -> Result<VfsImageFileEntry<I, L>, ErrorTrace> {
        match self {
            VfsImageFileEntry::Layer { .. } => {
                Err(keramics_core::error_trace_new!("No sub file entries"))
            }
            VfsImageFileEntry::Root { image } => {
                match image.get_layer_by_index(sub_file_entry_index) {
                    Ok(image_layer) => Ok(VfsImageFileEntry::Layer {
                        name_index: sub_file_entry_index,
                        layer: image_layer,
                    }),
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!("Unable to retrieve image layer: {}", sub_file_entry_index)
                        );
                        Err(error)
                    }
                }
            }
        }
    }

    /// Determines if the file entry is the root file entry.
    pub fn is_root_file_entry(&self) -> bool {
        matches!(self, VfsImageFileEntry::Root { .. })
    }
}
