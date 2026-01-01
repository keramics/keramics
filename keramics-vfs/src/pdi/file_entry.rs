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
use keramics_formats::pdi::{PdiImage, PdiImageLayer};

use crate::enums::VfsFileType;

/// Parallels Disk Image (PDI) storage media image file entry.
pub enum PdiFileEntry {
    /// Layer file entry.
    Layer {
        /// Layer index.
        index: usize,

        /// Layer.
        layer: Arc<RwLock<PdiImageLayer>>,

        /// Size.
        size: u64,
    },

    /// Root file entry.
    Root {
        /// Storage media image.
        image: Arc<PdiImage>,
    },
}

impl PdiFileEntry {
    /// Retrieves the default data stream.
    pub fn get_data_stream(&self) -> Result<Option<DataStreamReference>, ErrorTrace> {
        match self {
            PdiFileEntry::Layer { layer, .. } => Ok(Some(layer.clone())),
            PdiFileEntry::Root { .. } => Ok(None),
        }
    }

    /// Retrieves the file type.
    pub fn get_file_type(&self) -> VfsFileType {
        match self {
            PdiFileEntry::Layer { .. } => VfsFileType::File,
            PdiFileEntry::Root { .. } => VfsFileType::Directory,
        }
    }

    /// Retrieves the name.
    pub fn get_name(&self) -> PathComponent {
        match self {
            PdiFileEntry::Layer { index, .. } => PathComponent::from(format!("pdi{}", index + 1)),
            PdiFileEntry::Root { .. } => PathComponent::Root,
        }
    }

    /// Retrieves the size.
    pub fn get_size(&self) -> u64 {
        match self {
            PdiFileEntry::Layer { size, .. } => *size,
            PdiFileEntry::Root { .. } => 0,
        }
    }

    /// Retrieves the number of sub file entries.
    pub fn get_number_of_sub_file_entries(&self) -> usize {
        match self {
            PdiFileEntry::Layer { .. } => 0,
            PdiFileEntry::Root { image } => image.get_number_of_layers(),
        }
    }

    /// Retrieves a specific sub file entry.
    pub fn get_sub_file_entry_by_index(
        &self,
        sub_file_entry_index: usize,
    ) -> Result<PdiFileEntry, ErrorTrace> {
        match self {
            PdiFileEntry::Layer { .. } => {
                Err(keramics_core::error_trace_new!("No sub file entries"))
            }
            PdiFileEntry::Root { image } => match image.get_layer_by_index(sub_file_entry_index) {
                Ok(image_layer) => {
                    let media_size: u64 = match image_layer.read() {
                        Ok(vhd_file) => vhd_file.media_size,
                        Err(error) => {
                            return Err(keramics_core::error_trace_new_with_error!(
                                "Unable to obtain read lock on image layer",
                                error
                            ));
                        }
                    };
                    Ok(PdiFileEntry::Layer {
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
            PdiFileEntry::Layer { .. } => false,
            PdiFileEntry::Root { .. } => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_formats::{FileResolverReference, PathComponent, open_os_file_resolver};

    use crate::tests::get_test_data_path;

    fn get_image() -> Result<Arc<PdiImage>, ErrorTrace> {
        let mut image: PdiImage = PdiImage::new();

        let path_string: String = get_test_data_path("pdi/hfsplus.hdd");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let file_resolver: FileResolverReference = open_os_file_resolver(&path_buf)?;
        image.open(&file_resolver)?;

        Ok(Arc::new(image))
    }

    fn get_layer_file_entry(image: &Arc<PdiImage>) -> Result<PdiFileEntry, ErrorTrace> {
        let image_layer: Arc<RwLock<PdiImageLayer>> = image.get_layer_by_index(0)?;

        Ok(PdiFileEntry::Layer {
            index: 0,
            layer: image_layer,
            size: image.media_size,
        })
    }

    fn get_root_file_entry(image: &Arc<PdiImage>) -> PdiFileEntry {
        PdiFileEntry::Root {
            image: image.clone(),
        }
    }

    // TODO: add tests for get_data_stream

    #[test]
    fn test_get_file_type() -> Result<(), ErrorTrace> {
        let test_image: Arc<PdiImage> = get_image()?;

        let file_entry: PdiFileEntry = get_root_file_entry(&test_image);

        let file_type: VfsFileType = file_entry.get_file_type();
        assert_eq!(file_type, VfsFileType::Directory);

        let file_entry: PdiFileEntry = get_layer_file_entry(&test_image)?;

        let file_type: VfsFileType = file_entry.get_file_type();
        assert_eq!(file_type, VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_name() -> Result<(), ErrorTrace> {
        let test_image: Arc<PdiImage> = get_image()?;

        let file_entry: PdiFileEntry = get_root_file_entry(&test_image);

        let name: PathComponent = file_entry.get_name();
        assert_eq!(name, PathComponent::Root);

        let file_entry: PdiFileEntry = get_layer_file_entry(&test_image)?;

        let name: PathComponent = file_entry.get_name();
        assert_eq!(name, PathComponent::from("pdi1"));

        Ok(())
    }

    #[test]
    fn test_get_size() -> Result<(), ErrorTrace> {
        let test_image: Arc<PdiImage> = get_image()?;

        let file_entry: PdiFileEntry = get_root_file_entry(&test_image);

        let size: u64 = file_entry.get_size();
        assert_eq!(size, 0);

        let file_entry: PdiFileEntry = get_layer_file_entry(&test_image)?;

        let size: u64 = file_entry.get_size();
        assert_eq!(size, 33554432);

        Ok(())
    }

    #[test]
    fn test_get_number_of_sub_file_entries() -> Result<(), ErrorTrace> {
        let test_image: Arc<PdiImage> = get_image()?;

        let file_entry: PdiFileEntry = get_root_file_entry(&test_image);

        let number_of_sub_file_entries: usize = file_entry.get_number_of_sub_file_entries();
        assert_eq!(number_of_sub_file_entries, 1);

        let file_entry: PdiFileEntry = get_layer_file_entry(&test_image)?;

        let number_of_sub_file_entries: usize = file_entry.get_number_of_sub_file_entries();
        assert_eq!(number_of_sub_file_entries, 0);

        Ok(())
    }

    #[test]
    fn test_get_sub_file_entry_by_index() -> Result<(), ErrorTrace> {
        let test_image: Arc<PdiImage> = get_image()?;

        let file_entry: PdiFileEntry = get_root_file_entry(&test_image);

        let sub_file_entry: PdiFileEntry = file_entry.get_sub_file_entry_by_index(0)?;

        let name: PathComponent = sub_file_entry.get_name();
        assert_eq!(name, PathComponent::from("pdi1"));

        let result: Result<PdiFileEntry, ErrorTrace> = file_entry.get_sub_file_entry_by_index(99);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_is_root_file_entry() -> Result<(), ErrorTrace> {
        let test_image: Arc<PdiImage> = get_image()?;

        let file_entry: PdiFileEntry = get_root_file_entry(&test_image);
        assert_eq!(file_entry.is_root_file_entry(), true);

        let file_entry: PdiFileEntry = get_layer_file_entry(&test_image)?;
        assert_eq!(file_entry.is_root_file_entry(), false);

        Ok(())
    }
}
