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

use std::collections::HashMap;
use std::sync::Arc;

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_types::Uuid;

use crate::file_resolver::FileResolverReference;
use crate::path_component::PathComponent;

use super::file::VdiFile;

pub type VdiImageLayer = Arc<VdiFile>;

/// Virtual Disk Image (VDI) storage media image.
pub struct VdiImage {
    /// Layers.
    layers: Vec<Arc<VdiFile>>,

    /// Bytes per sector.
    bytes_per_sector: u16,
}

impl VdiImage {
    /// Creates a new storage media image.
    pub fn new() -> Self {
        Self {
            layers: Vec::new(),
            bytes_per_sector: 0,
        }
    }

    /// Retrieves the bytes per sector.
    pub fn get_bytes_per_sector(&self) -> u16 {
        self.bytes_per_sector
    }

    /// Retrieves the number of layers.
    pub fn get_number_of_layers(&self) -> usize {
        self.layers.len()
    }

    /// Retrieves a layer by index.
    pub fn get_layer_by_index(&self, layer_index: usize) -> Result<VdiImageLayer, ErrorTrace> {
        match self.layers.get(layer_index) {
            Some(file) => Ok(file.clone()),
            None => Err(keramics_core::error_trace_new!(format!(
                "No layer with index: {}",
                layer_index
            ))),
        }
    }

    /// Opens a storage media image.
    pub fn open(
        &mut self,
        file_resolver: &FileResolverReference,
        file_names: &[PathComponent],
    ) -> Result<(), ErrorTrace> {
        let mut files_per_identifier: HashMap<Uuid, VdiFile> = HashMap::new();
        let mut first_identifier: Option<Uuid> = None;

        for (file_index, file_name) in file_names.iter().enumerate() {
            let path_components: [PathComponent; 1] = [file_name.clone()];

            let data_stream: DataStreamReference =
                match file_resolver.get_data_stream(&path_components) {
                    Ok(Some(data_stream)) => data_stream,
                    Ok(None) => {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Missing data stream: {}",
                            file_name
                        )));
                    }
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!("Unable to open file: {}", file_name)
                        );
                        return Err(error);
                    }
                };
            let mut file: VdiFile = VdiFile::new();

            match file.read_data_stream(&data_stream) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to read file");
                    return Err(error);
                }
            }
            let identifier: &Uuid = file.get_identifier();

            if file_index == 0 {
                first_identifier = Some(identifier.clone());
            }
            files_per_identifier.insert(identifier.clone(), file);
        }
        let identifier: Uuid = match first_identifier {
            Some(identifier) => identifier,
            None => {
                return Err(keramics_core::error_trace_new!("Missing image identifier"));
            }
        };
        let mut file: VdiFile = match files_per_identifier.remove(&identifier) {
            Some(file) => file,
            None => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Missing file with identifier: {}",
                    identifier
                )));
            }
        };
        let mut files: Vec<VdiFile> = Vec::new();

        while let Some(parent_identifier) = file.get_parent_identifier() {
            file = match files_per_identifier.remove(parent_identifier) {
                Some(parent_file) => {
                    files.push(file);

                    parent_file
                }
                None => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Missing parent file with identifier: {}",
                        parent_identifier
                    )));
                }
            };
        }
        files.push(file);

        let mut file_index: usize = 0;
        while let Some(mut file) = files.pop() {
            if file_index > 0 {
                match file.set_parent(&self.layers[file_index - 1]) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to set parent");
                        return Err(error);
                    }
                }
            }
            self.layers.push(Arc::new(file));

            file_index += 1;
        }
        self.bytes_per_sector = 512;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use crate::os_file_resolver::OsFileResolver;

    use crate::tests::get_test_data_path;

    fn get_image() -> Result<VdiImage, ErrorTrace> {
        // TODO: create differential test image
        let mut image: VdiImage = VdiImage::new();

        let path_string: String = get_test_data_path("vdi");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let file_resolver: FileResolverReference =
            FileResolverReference::new(Box::new(OsFileResolver::new(path_buf)));
        let file_names: [PathComponent; 1] = [PathComponent::from("ext2.vdi")];
        image.open(&file_resolver, &file_names)?;

        Ok(image)
    }

    #[test]
    fn test_get_bytes_per_sector() -> Result<(), ErrorTrace> {
        let image: VdiImage = get_image()?;

        let bytes_per_sector: u16 = image.get_bytes_per_sector();
        assert_eq!(bytes_per_sector, 512);

        Ok(())
    }

    #[test]
    fn test_get_number_of_layers() -> Result<(), ErrorTrace> {
        let image: VdiImage = get_image()?;

        let number_of_layers: usize = image.get_number_of_layers();
        assert_eq!(number_of_layers, 1);

        Ok(())
    }

    #[test]
    fn test_get_layer_by_index() -> Result<(), ErrorTrace> {
        let image: VdiImage = get_image()?;

        let image_layer: VdiImageLayer = image.get_layer_by_index(0)?;

        assert_eq!(image_layer.media_size, 4194304);
        assert_eq!(
            image_layer.identifier.to_string(),
            "88437ae8-9631-4c48-81c3-e787cc586501"
        );
        Ok(())
    }

    #[test]
    fn test_open() -> Result<(), ErrorTrace> {
        // TODO: create differential test image
        let mut image: VdiImage = VdiImage::new();

        let path_string: String = get_test_data_path("vdi");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let file_resolver: FileResolverReference =
            FileResolverReference::new(Box::new(OsFileResolver::new(path_buf)));
        let file_names: [PathComponent; 1] = [PathComponent::from("ext2.vdi")];
        image.open(&file_resolver, &file_names)?;

        assert_eq!(image.layers.len(), 1);
        assert_eq!(image.bytes_per_sector, 512);

        Ok(())
    }
}
