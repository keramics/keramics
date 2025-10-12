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
use keramics_formats::qcow::{QcowImage, QcowImageLayer};

use crate::enums::VfsFileType;

/// QEMU Copy-On-Write (QCOW) storage media image file entry.
pub enum QcowFileEntry {
    /// Layer file entry.
    Layer {
        /// Layer index.
        index: usize,

        /// Layer.
        layer: QcowImageLayer,
    },

    /// Root file entry.
    Root {
        /// Storage media image.
        image: Arc<QcowImage>,
    },
}

impl QcowFileEntry {
    /// Retrieves the default data stream.
    pub fn get_data_stream(&self) -> Result<Option<DataStreamReference>, ErrorTrace> {
        match self {
            QcowFileEntry::Layer { layer, .. } => Ok(Some(layer.clone())),
            QcowFileEntry::Root { .. } => Ok(None),
        }
    }

    /// Retrieves the file type.
    pub fn get_file_type(&self) -> VfsFileType {
        match self {
            QcowFileEntry::Layer { .. } => VfsFileType::File,
            QcowFileEntry::Root { .. } => VfsFileType::Directory,
        }
    }

    /// Retrieves the name.
    pub fn get_name(&self) -> Option<String> {
        match self {
            QcowFileEntry::Layer { index, .. } => Some(format!("qcow{}", index + 1)),
            QcowFileEntry::Root { .. } => None,
        }
    }

    /// Retrieves the number of sub file entries.
    pub fn get_number_of_sub_file_entries(&mut self) -> Result<usize, ErrorTrace> {
        match self {
            QcowFileEntry::Layer { .. } => Ok(0),
            QcowFileEntry::Root { image } => Ok(image.get_number_of_layers()),
        }
    }

    /// Retrieves a specific sub file entry.
    pub fn get_sub_file_entry_by_index(
        &mut self,
        sub_file_entry_index: usize,
    ) -> Result<QcowFileEntry, ErrorTrace> {
        match self {
            QcowFileEntry::Layer { .. } => {
                Err(keramics_core::error_trace_new!("No sub file entries"))
            }
            QcowFileEntry::Root { image } => match image.get_layer_by_index(sub_file_entry_index) {
                Ok(qcow_layer) => Ok(QcowFileEntry::Layer {
                    index: sub_file_entry_index,
                    layer: qcow_layer.clone(),
                }),
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to retrieve QCOW image layer: {}",
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

    use std::path::PathBuf;

    use keramics_formats::{FileResolverReference, PathComponent, open_os_file_resolver};

    fn get_image() -> Result<QcowImage, ErrorTrace> {
        let mut image: QcowImage = QcowImage::new();

        let path_buf: PathBuf = PathBuf::from("../test_data/qcow");
        let file_resolver: FileResolverReference = open_os_file_resolver(&path_buf)?;
        let file_name: PathComponent = PathComponent::from("ext2.qcow2");
        image.open(&file_resolver, &file_name)?;

        Ok(image)
    }

    // TODO: add tests for get_data_stream

    #[test]
    fn test_get_file_type() -> Result<(), ErrorTrace> {
        let qcow_image: Arc<QcowImage> = Arc::new(get_image()?);

        let file_entry = QcowFileEntry::Root {
            image: qcow_image.clone(),
        };

        let file_type: VfsFileType = file_entry.get_file_type();
        assert!(file_type == VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_name() -> Result<(), ErrorTrace> {
        let qcow_image: Arc<QcowImage> = Arc::new(get_image()?);

        let file_entry = QcowFileEntry::Root {
            image: qcow_image.clone(),
        };

        let name: Option<String> = file_entry.get_name();
        assert!(name.is_none());

        let qcow_layer: QcowImageLayer = qcow_image.get_layer_by_index(0)?;
        let file_entry = QcowFileEntry::Layer {
            index: 0,
            layer: qcow_layer.clone(),
        };

        let name: Option<String> = file_entry.get_name();
        assert_eq!(name, Some(String::from("qcow1")));

        Ok(())
    }

    // TODO: add tests for get_number_of_sub_file_entries
    // TODO: add tests for get_sub_file_entry_by_index
}
