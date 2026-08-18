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
use keramics_formats::vhdx::{VhdxImage, VhdxImageLayer};
use keramics_types::Uuid;

use crate::enums::VfsFileType;

/// Virtual Hard Disk version 2 (VHDX) storage media image file entry.
pub enum VhdxFileEntry {
    /// Layer file entry.
    Layer {
        /// Layer index.
        index: usize,

        /// Layer.
        layer: VhdxImageLayer,

        /// Size.
        size: u64,

        /// Identifier.
        identifier: Uuid,
    },

    /// Root file entry.
    Root {
        /// Storage media image.
        image: Arc<VhdxImage>,
    },
}

impl VhdxFileEntry {
    /// Retrieves the default data stream.
    pub fn get_data_stream(&self) -> Result<Option<DataStreamReference>, ErrorTrace> {
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

    /// Retrieves the identifier.
    pub fn get_identifier(&self) -> Option<&Uuid> {
        match self {
            VhdxFileEntry::Layer { identifier, .. } => Some(&identifier),
            VhdxFileEntry::Root { .. } => None,
        }
    }

    /// Retrieves the (image) layer number.
    pub fn get_layer_number(&self) -> Option<usize> {
        match self {
            VhdxFileEntry::Layer { index, .. } => Some(index + 1),
            VhdxFileEntry::Root { .. } => None,
        }
    }

    /// Retrieves the name.
    pub fn get_name(&self) -> PathComponent {
        match self {
            VhdxFileEntry::Layer { index, .. } => PathComponent::from(format!("vhdx{}", index + 1)),
            VhdxFileEntry::Root { .. } => PathComponent::Root,
        }
    }

    /// Retrieves the size.
    pub fn get_size(&self) -> u64 {
        match self {
            VhdxFileEntry::Layer { size, .. } => *size,
            VhdxFileEntry::Root { .. } => 0,
        }
    }

    /// Retrieves the number of sub file entries.
    pub fn get_number_of_sub_file_entries(&self) -> usize {
        match self {
            VhdxFileEntry::Layer { .. } => 0,
            VhdxFileEntry::Root { image } => image.get_number_of_layers(),
        }
    }

    /// Retrieves a specific sub file entry.
    pub fn get_sub_file_entry_by_index(
        &self,
        sub_file_entry_index: usize,
    ) -> Result<VhdxFileEntry, ErrorTrace> {
        match self {
            VhdxFileEntry::Layer { .. } => {
                Err(keramics_core::error_trace_new!("No sub file entries"))
            }
            VhdxFileEntry::Root { image } => match image.get_layer_by_index(sub_file_entry_index) {
                Ok(image_layer) => {
                    let media_size: u64;
                    let identifier: Uuid;

                    match image_layer.read() {
                        Ok(vhdx_image_layer) => {
                            media_size = vhdx_image_layer.get_media_size();
                            identifier = vhdx_image_layer.get_identifier().clone();
                        }
                        Err(error) => {
                            return Err(keramics_core::error_trace_new_with_error!(
                                format!(
                                    "Unable to obtain read lock on image layer: {}",
                                    sub_file_entry_index
                                ),
                                error
                            ));
                        }
                    }
                    Ok(VhdxFileEntry::Layer {
                        index: sub_file_entry_index,
                        layer: image_layer.clone(),
                        size: media_size,
                        identifier,
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
            VhdxFileEntry::Layer { .. } => false,
            VhdxFileEntry::Root { .. } => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_formats::{FileResolverReference, PathComponent, open_os_file_resolver};

    use crate::tests::get_test_data_path;

    fn get_image() -> Result<Arc<VhdxImage>, ErrorTrace> {
        let mut image: VhdxImage = VhdxImage::new();

        let path_string: String = get_test_data_path("vhdx");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let file_resolver: FileResolverReference = open_os_file_resolver(&path_buf)?;
        let file_name: PathComponent = PathComponent::from("ntfs-differential.vhdx");
        image.open(&file_resolver, &file_name)?;

        Ok(Arc::new(image))
    }

    fn get_layer_file_entry(image: &Arc<VhdxImage>) -> Result<VhdxFileEntry, ErrorTrace> {
        let image_layer: VhdxImageLayer = image.get_layer_by_index(0)?;

        let media_size: u64;
        let identifier: Uuid;

        match image_layer.read() {
            Ok(vhdx_image_layer) => {
                media_size = vhdx_image_layer.get_media_size();
                identifier = vhdx_image_layer.get_identifier().clone();
            }
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to obtain read lock on image layer",
                    error
                ));
            }
        }
        Ok(VhdxFileEntry::Layer {
            index: 0,
            layer: image_layer.clone(),
            size: media_size,
            identifier,
        })
    }

    fn get_root_file_entry(image: &Arc<VhdxImage>) -> VhdxFileEntry {
        VhdxFileEntry::Root {
            image: image.clone(),
        }
    }

    // TODO: add tests for get_data_stream

    #[test]
    fn test_get_file_type() -> Result<(), ErrorTrace> {
        let test_image: Arc<VhdxImage> = get_image()?;

        let file_entry: VhdxFileEntry = get_root_file_entry(&test_image);

        let file_type: VfsFileType = file_entry.get_file_type();
        assert_eq!(file_type, VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_get_identifier() -> Result<(), ErrorTrace> {
        let test_image: Arc<VhdxImage> = get_image()?;

        let file_entry: VhdxFileEntry = get_root_file_entry(&test_image);
        let result: Option<&Uuid> = file_entry.get_identifier();
        assert!(result.is_none());

        let file_entry: VhdxFileEntry = get_layer_file_entry(&test_image)?;
        let identifier: &Uuid = file_entry.get_identifier().unwrap();
        assert_eq!(
            identifier.to_string(),
            "7584f8fb-36d3-4091-afb5-b1afe587bfa8"
        );
        Ok(())
    }

    #[test]
    fn test_get_layer_number() -> Result<(), ErrorTrace> {
        let test_image: Arc<VhdxImage> = get_image()?;

        let file_entry: VhdxFileEntry = get_root_file_entry(&test_image);

        let layer_number: Option<usize> = file_entry.get_layer_number();
        assert_eq!(layer_number, None);

        let file_entry: VhdxFileEntry = get_layer_file_entry(&test_image)?;

        let layer_number: Option<usize> = file_entry.get_layer_number();
        assert_eq!(layer_number, Some(1));

        Ok(())
    }

    #[test]
    fn test_get_name() -> Result<(), ErrorTrace> {
        let test_image: Arc<VhdxImage> = get_image()?;

        let file_entry: VhdxFileEntry = get_root_file_entry(&test_image);

        let name: PathComponent = file_entry.get_name();
        assert_eq!(name, PathComponent::Root);

        let file_entry: VhdxFileEntry = get_layer_file_entry(&test_image)?;

        let name: PathComponent = file_entry.get_name();
        assert_eq!(name, PathComponent::from("vhdx1"));

        Ok(())
    }

    #[test]
    fn test_get_size() -> Result<(), ErrorTrace> {
        let test_image: Arc<VhdxImage> = get_image()?;

        let file_entry: VhdxFileEntry = get_root_file_entry(&test_image);

        let size: u64 = file_entry.get_size();
        assert_eq!(size, 0);

        let file_entry: VhdxFileEntry = get_layer_file_entry(&test_image)?;

        let size: u64 = file_entry.get_size();
        assert_eq!(size, 4194304);

        Ok(())
    }

    #[test]
    fn test_get_number_of_sub_file_entries() -> Result<(), ErrorTrace> {
        let test_image: Arc<VhdxImage> = get_image()?;

        let file_entry: VhdxFileEntry = get_root_file_entry(&test_image);

        let number_of_sub_file_entries: usize = file_entry.get_number_of_sub_file_entries();
        assert_eq!(number_of_sub_file_entries, 2);

        let file_entry: VhdxFileEntry = get_layer_file_entry(&test_image)?;

        let number_of_sub_file_entries: usize = file_entry.get_number_of_sub_file_entries();
        assert_eq!(number_of_sub_file_entries, 0);

        Ok(())
    }

    #[test]
    fn test_get_sub_file_entry_by_index() -> Result<(), ErrorTrace> {
        let test_image: Arc<VhdxImage> = get_image()?;

        let file_entry: VhdxFileEntry = get_root_file_entry(&test_image);

        let sub_file_entry: VhdxFileEntry = file_entry.get_sub_file_entry_by_index(0)?;

        let name: PathComponent = sub_file_entry.get_name();
        assert_eq!(name, PathComponent::from("vhdx1"));

        let result: Result<VhdxFileEntry, ErrorTrace> = file_entry.get_sub_file_entry_by_index(99);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_is_root_file_entry() -> Result<(), ErrorTrace> {
        let test_image: Arc<VhdxImage> = get_image()?;

        let file_entry: VhdxFileEntry = get_root_file_entry(&test_image);

        assert_eq!(file_entry.is_root_file_entry(), true);

        let file_entry: VhdxFileEntry = get_layer_file_entry(&test_image)?;

        assert_eq!(file_entry.is_root_file_entry(), false);

        Ok(())
    }
}
