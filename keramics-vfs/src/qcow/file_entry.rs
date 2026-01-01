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

        /// Size.
        size: u64,
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
    pub fn get_name(&self) -> PathComponent {
        match self {
            QcowFileEntry::Layer { index, .. } => PathComponent::from(format!("qcow{}", index + 1)),
            QcowFileEntry::Root { .. } => PathComponent::Root,
        }
    }

    /// Retrieves the size.
    pub fn get_size(&self) -> u64 {
        match self {
            QcowFileEntry::Layer { size, .. } => *size,
            QcowFileEntry::Root { .. } => 0,
        }
    }

    /// Retrieves the number of sub file entries.
    pub fn get_number_of_sub_file_entries(&self) -> usize {
        match self {
            QcowFileEntry::Layer { .. } => 0,
            QcowFileEntry::Root { image } => image.get_number_of_layers(),
        }
    }

    /// Retrieves a specific sub file entry.
    pub fn get_sub_file_entry_by_index(
        &self,
        sub_file_entry_index: usize,
    ) -> Result<QcowFileEntry, ErrorTrace> {
        match self {
            QcowFileEntry::Layer { .. } => {
                Err(keramics_core::error_trace_new!("No sub file entries"))
            }
            QcowFileEntry::Root { image } => match image.get_layer_by_index(sub_file_entry_index) {
                Ok(image_layer) => {
                    let media_size: u64 = match image_layer.read() {
                        Ok(qcow_file) => qcow_file.media_size,
                        Err(error) => {
                            return Err(keramics_core::error_trace_new_with_error!(
                                "Unable to obtain read lock on image layer",
                                error
                            ));
                        }
                    };
                    Ok(QcowFileEntry::Layer {
                        index: sub_file_entry_index,
                        layer: image_layer.clone(),
                        size: media_size,
                    })
                }
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to retrieve image layer: {}", sub_file_entry_index)
                    );
                    return Err(error);
                }
            },
        }
    }

    /// Determines if the file entry is the root file entry.
    pub fn is_root_file_entry(&self) -> bool {
        match self {
            QcowFileEntry::Layer { .. } => false,
            QcowFileEntry::Root { .. } => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_formats::{FileResolverReference, PathComponent, open_os_file_resolver};

    use crate::tests::get_test_data_path;

    fn get_image() -> Result<QcowImage, ErrorTrace> {
        let mut image: QcowImage = QcowImage::new();

        let path_string: String = get_test_data_path("qcow");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let file_resolver: FileResolverReference = open_os_file_resolver(&path_buf)?;
        let file_name: PathComponent = PathComponent::from("ext2.qcow2");
        image.open(&file_resolver, &file_name)?;

        Ok(image)
    }

    // TODO: add tests for get_data_stream

    #[test]
    fn test_get_file_type() -> Result<(), ErrorTrace> {
        let qcow_image: QcowImage = get_image()?;

        let test_image: Arc<QcowImage> = Arc::new(qcow_image);

        let file_entry = QcowFileEntry::Root {
            image: test_image.clone(),
        };

        let file_type: VfsFileType = file_entry.get_file_type();
        assert_eq!(file_type, VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_get_name() -> Result<(), ErrorTrace> {
        let qcow_image: QcowImage = get_image()?;

        let test_image: Arc<QcowImage> = Arc::new(qcow_image);

        let file_entry = QcowFileEntry::Root {
            image: test_image.clone(),
        };

        let name: PathComponent = file_entry.get_name();
        assert_eq!(name, PathComponent::Root);

        let qcow_image_layer: QcowImageLayer = test_image.get_layer_by_index(0)?;
        let file_entry = QcowFileEntry::Layer {
            index: 0,
            layer: qcow_image_layer.clone(),
            size: 4194304,
        };

        let name: PathComponent = file_entry.get_name();
        assert_eq!(name, PathComponent::from("qcow1"));

        Ok(())
    }

    #[test]
    fn test_get_size() -> Result<(), ErrorTrace> {
        let qcow_image: QcowImage = get_image()?;

        let test_image: Arc<QcowImage> = Arc::new(qcow_image);

        let file_entry = QcowFileEntry::Root {
            image: test_image.clone(),
        };

        let size: u64 = file_entry.get_size();
        assert_eq!(size, 0);

        let qcow_image_layer: QcowImageLayer = test_image.get_layer_by_index(0)?;
        let file_entry = QcowFileEntry::Layer {
            index: 0,
            layer: qcow_image_layer.clone(),
            size: 4194304,
        };

        let size: u64 = file_entry.get_size();
        assert_eq!(size, 4194304);

        Ok(())
    }

    #[test]
    fn test_get_number_of_sub_file_entries() -> Result<(), ErrorTrace> {
        let qcow_image: QcowImage = get_image()?;

        let test_image: Arc<QcowImage> = Arc::new(qcow_image);

        let file_entry = QcowFileEntry::Root {
            image: test_image.clone(),
        };

        let number_of_sub_file_entries: usize = file_entry.get_number_of_sub_file_entries();
        assert_eq!(number_of_sub_file_entries, 1);

        let qcow_image_layer: QcowImageLayer = test_image.get_layer_by_index(0)?;
        let file_entry = QcowFileEntry::Layer {
            index: 0,
            layer: qcow_image_layer.clone(),
            size: 4194304,
        };

        let number_of_sub_file_entries: usize = file_entry.get_number_of_sub_file_entries();
        assert_eq!(number_of_sub_file_entries, 0);

        Ok(())
    }

    #[test]
    fn test_get_sub_file_entry_by_index() -> Result<(), ErrorTrace> {
        let qcow_image: QcowImage = get_image()?;

        let test_image: Arc<QcowImage> = Arc::new(qcow_image);

        let file_entry = QcowFileEntry::Root {
            image: test_image.clone(),
        };

        let sub_file_entry: QcowFileEntry = file_entry.get_sub_file_entry_by_index(0)?;

        let name: PathComponent = sub_file_entry.get_name();
        assert_eq!(name, PathComponent::from("qcow1"));

        let result: Result<QcowFileEntry, ErrorTrace> = file_entry.get_sub_file_entry_by_index(99);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_is_root_file_entry() -> Result<(), ErrorTrace> {
        let qcow_image: QcowImage = get_image()?;

        let test_image: Arc<QcowImage> = Arc::new(qcow_image);

        let file_entry = QcowFileEntry::Root {
            image: test_image.clone(),
        };

        assert_eq!(file_entry.is_root_file_entry(), true);

        let qcow_image_layer: QcowImageLayer = test_image.get_layer_by_index(0)?;
        let file_entry = QcowFileEntry::Layer {
            index: 0,
            layer: qcow_image_layer.clone(),
            size: 4194304,
        };

        assert_eq!(file_entry.is_root_file_entry(), false);

        Ok(())
    }
}
