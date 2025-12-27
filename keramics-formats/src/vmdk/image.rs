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

use std::sync::{Arc, RwLock};

use keramics_core::ErrorTrace;

use crate::file_resolver::FileResolverReference;
use crate::path_component::PathComponent;

use super::image_layer::VmdkImageLayer;

/// VMware Virtual Disk (VMDK) storage media image.
pub struct VmdkImage {
    /// Layers.
    layers: Vec<Arc<RwLock<VmdkImageLayer>>>,

    /// Bytes per sector.
    pub bytes_per_sector: u16,

    /// Media size.
    pub media_size: u64,
}

impl VmdkImage {
    /// Creates a new storage media image.
    pub fn new() -> Self {
        Self {
            layers: Vec::new(),
            bytes_per_sector: 0,
            media_size: 0,
        }
    }

    /// Retrieves the number of layers.
    pub fn get_number_of_layers(&self) -> usize {
        self.layers.len()
    }

    /// Retrieves a layer by index.
    pub fn get_layer_by_index(
        &self,
        layer_index: usize,
    ) -> Result<Arc<RwLock<VmdkImageLayer>>, ErrorTrace> {
        match self.layers.get(layer_index) {
            Some(image_layer) => Ok(image_layer.clone()),
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
        file_name: &PathComponent,
    ) -> Result<(), ErrorTrace> {
        let mut image_layers: Vec<VmdkImageLayer> = Vec::new();

        let mut image_layer: VmdkImageLayer = VmdkImageLayer::new();
        match image_layer.open(file_resolver, &file_name) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read top image layer");
                return Err(error);
            }
        }
        self.bytes_per_sector = image_layer.bytes_per_sector;
        self.media_size = image_layer.media_size;

        while image_layer.parent_content_identifier.is_some() {
            let parent_file_name: PathComponent = match &image_layer.parent_name {
                Some(file_name) => PathComponent::from(file_name),
                None => {
                    return Err(keramics_core::error_trace_new!("Missing parent file name"));
                }
            };
            let mut parent_image_layer: VmdkImageLayer = VmdkImageLayer::new();
            match parent_image_layer.open(file_resolver, &parent_file_name) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to read parent image layer"
                    );
                    return Err(error);
                }
            }
            image_layers.push(image_layer);

            image_layer = parent_image_layer;
        }
        image_layers.push(image_layer);

        let mut image_layer_index: usize = 0;
        while let Some(mut image_layer) = image_layers.pop() {
            if image_layer_index > 0 {
                match image_layer.set_parent(&mut self.layers[image_layer_index - 1]) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to set parent");
                        return Err(error);
                    }
                }
            }
            self.layers.push(Arc::new(RwLock::new(image_layer)));

            image_layer_index += 1;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use crate::os_file_resolver::open_os_file_resolver;

    use crate::tests::get_test_data_path;

    fn get_image() -> Result<VmdkImage, ErrorTrace> {
        let mut image: VmdkImage = VmdkImage::new();

        let path_string: String = get_test_data_path("vmdk");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let file_resolver: FileResolverReference = open_os_file_resolver(&path_buf)?;
        let file_name: PathComponent = PathComponent::from("ext2.vmdk");
        image.open(&file_resolver, &file_name)?;

        Ok(image)
    }

    // TODO: add tests for get_number_of_layers
    // TODO: add tests for get_layer_by_index

    #[test]
    fn test_open() -> Result<(), ErrorTrace> {
        let mut image: VmdkImage = VmdkImage::new();

        let path_string: String = get_test_data_path("vmdk");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let file_resolver: FileResolverReference = open_os_file_resolver(&path_buf)?;
        let file_name: PathComponent = PathComponent::from("ext2.vmdk");
        image.open(&file_resolver, &file_name)?;

        Ok(())
    }
}
