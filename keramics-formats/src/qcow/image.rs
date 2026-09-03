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

use crate::file_resolver::FileResolverReference;
use crate::path_component::PathComponent;

use super::credential::QcowCredential;
use super::file::QcowFile;

pub type QcowImageLayer = Arc<QcowFile>;

/// QEMU Copy-On-Write (QCOW) storage media image.
pub struct QcowImage {
    /// Layers.
    layers: Vec<Arc<QcowFile>>,

    /// Bytes per sector.
    bytes_per_sector: u16,

    /// Value to indicate the (encrypted) image is locked.
    is_locked: bool,
}

impl QcowImage {
    /// Creates a new storage media image.
    pub fn new() -> Self {
        Self {
            layers: Vec::new(),
            bytes_per_sector: 0,
            is_locked: false,
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
    pub fn get_layer_by_index(&self, layer_index: usize) -> Result<QcowImageLayer, ErrorTrace> {
        match self.layers.get(layer_index) {
            Some(file) => Ok(file.clone()),
            None => Err(keramics_core::error_trace_new!(format!(
                "No layer with index: {}",
                layer_index
            ))),
        }
    }

    /// Determines if the (encrypted) image is locked.
    pub fn is_locked(&self) -> bool {
        self.is_locked
    }

    /// Opens a storage media image.
    pub fn open(
        &mut self,
        file_resolver: &FileResolverReference,
        file_name: &PathComponent,
    ) -> Result<(), ErrorTrace> {
        let path_components: [PathComponent; 1] = [file_name.clone()];

        let data_stream: DataStreamReference = match file_resolver.get_data_stream(&path_components)
        {
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
        let mut files: Vec<QcowFile> = Vec::new();
        let mut file: QcowFile = QcowFile::new();

        match file.read_data_stream(&data_stream) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read file");
                return Err(error);
            }
        }
        self.bytes_per_sector = file.bytes_per_sector;
        self.is_locked = file.is_locked();

        while let Some(file_name) = file.get_backing_file_name() {
            let backing_file_name: String = file_name.to_string();

            let path_components: [PathComponent; 1] = [PathComponent::from(&backing_file_name)];
            let data_stream: DataStreamReference =
                match file_resolver.get_data_stream(&path_components) {
                    Ok(Some(data_stream)) => data_stream,
                    Ok(None) => {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Missing backing file: {}",
                            backing_file_name
                        )));
                    }
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to open backing file");
                        return Err(error);
                    }
                };
            let mut backing_file: QcowFile = QcowFile::new();

            match backing_file.read_data_stream(&data_stream) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to read backing file");
                    return Err(error);
                }
            }
            files.push(file);

            file = backing_file;
        }
        files.push(file);

        let mut file_index: usize = 0;
        while let Some(mut file) = files.pop() {
            if file_index > 0 {
                match file.set_backing_file(&mut self.layers[file_index - 1]) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to set backing file");
                        return Err(error);
                    }
                }
            }
            self.layers.push(Arc::new(file));

            file_index += 1;
        }
        Ok(())
    }

    /// Unlocks a locked (encrypted) image.
    pub fn unlock(&mut self, credentials: &[QcowCredential]) -> Result<bool, ErrorTrace> {
        if !self.is_locked {
            return Ok(true);
        }
        let layer: &mut Arc<QcowFile> = match self.layers.last_mut() {
            Some(layer) => layer,
            None => return Err(keramics_core::error_trace_new!("Missing upper layer")),
        };
        match Arc::get_mut(&mut *layer) {
            Some(file) => {
                let result: bool = match file.unlock(credentials) {
                    Ok(result) => result,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to unlock QCOW file");
                        return Err(error);
                    }
                };
                if result {
                    self.is_locked = false;
                }
            }
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Unable to obtain mutable reference to QCOW file"
                ));
            }
        }
        Ok(!self.is_locked)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use crate::os_file_resolver::open_os_file_resolver;

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

    #[test]
    fn test_get_bytes_per_sector() -> Result<(), ErrorTrace> {
        let image: QcowImage = get_image()?;

        let bytes_per_sector: u16 = image.get_bytes_per_sector();
        assert_eq!(bytes_per_sector, 512);

        Ok(())
    }

    #[test]
    fn test_get_number_of_layers() -> Result<(), ErrorTrace> {
        let image: QcowImage = get_image()?;

        let number_of_layers: usize = image.get_number_of_layers();
        assert_eq!(number_of_layers, 1);

        Ok(())
    }

    #[test]
    fn test_get_layer_by_index() -> Result<(), ErrorTrace> {
        let image: QcowImage = get_image()?;

        let image_layer: QcowImageLayer = image.get_layer_by_index(0)?;
        assert_eq!(image_layer.media_size, 4194304);

        Ok(())
    }

    #[test]
    fn test_open() -> Result<(), ErrorTrace> {
        let mut image: QcowImage = QcowImage::new();

        let path_string: String = get_test_data_path("qcow");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let file_resolver: FileResolverReference = open_os_file_resolver(&path_buf)?;
        let file_name: PathComponent = PathComponent::from("ext2.qcow2");
        image.open(&file_resolver, &file_name)?;

        assert_eq!(image.layers.len(), 1);
        assert_eq!(image.bytes_per_sector, 512);

        Ok(())
    }
}
