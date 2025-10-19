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

use keramics_core::{DataStreamReference, ErrorTrace};

use crate::file_resolver::FileResolverReference;
use crate::path_component::PathComponent;

use super::file::QcowFile;

pub type QcowImageLayer = Arc<RwLock<QcowFile>>;

/// QEMU Copy-On-Write (QCOW) storage media image.
pub struct QcowImage {
    /// Files.
    files: Vec<Arc<RwLock<QcowFile>>>,
}

impl QcowImage {
    /// Creates a new storage media image.
    pub fn new() -> Self {
        Self { files: Vec::new() }
    }

    /// Retrieves the number of layers.
    pub fn get_number_of_layers(&self) -> usize {
        self.files.len()
    }

    /// Retrieves a layer by index.
    pub fn get_layer_by_index(&self, layer_index: usize) -> Result<QcowImageLayer, ErrorTrace> {
        match self.files.get(layer_index) {
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
        file_name: &PathComponent,
    ) -> Result<(), ErrorTrace> {
        let mut files: Vec<QcowFile> = Vec::new();

        let path_components: [PathComponent; 1] = [file_name.clone()];
        let result: Option<DataStreamReference> =
            match file_resolver.get_data_stream(&path_components) {
                Ok(result) => result,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to open file: {}", file_name.to_string())
                    );
                    return Err(error);
                }
            };
        let data_stream: DataStreamReference = match result {
            Some(data_stream) => data_stream,
            None => {
                return Err(keramics_core::error_trace_new!(format!(
                    "No such file: {}",
                    file_name.to_string()
                )));
            }
        };
        let mut file: QcowFile = QcowFile::new();

        match file.read_data_stream(&data_stream) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read file");
                return Err(error);
            }
        }
        while let Some(file_name) = file.get_backing_file_name() {
            let backing_file_name: String = file_name.to_string();

            let path_components: [PathComponent; 1] = [PathComponent::from(&backing_file_name)];
            let result: Option<DataStreamReference> =
                match file_resolver.get_data_stream(&path_components) {
                    Ok(result) => result,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to open backing file");
                        return Err(error);
                    }
                };
            let data_stream: DataStreamReference = match result {
                Some(data_stream) => data_stream,
                None => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Missing backing file: {}",
                        backing_file_name
                    )));
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
                match file.set_backing_file(&mut self.files[file_index - 1]) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to set backing file");
                        return Err(error);
                    }
                }
            }
            self.files.push(Arc::new(RwLock::new(file)));

            file_index += 1;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use crate::os_file_resolver::open_os_file_resolver;

    fn get_image() -> Result<QcowImage, ErrorTrace> {
        let mut image: QcowImage = QcowImage::new();

        let path_buf: PathBuf = PathBuf::from("../test_data/qcow");
        let file_resolver: FileResolverReference = open_os_file_resolver(&path_buf)?;
        let file_name: PathComponent = PathComponent::from("ext2.qcow2");
        image.open(&file_resolver, &file_name)?;

        Ok(image)
    }

    #[test]
    fn test_get_number_of_layers() -> Result<(), ErrorTrace> {
        let image: QcowImage = get_image()?;

        assert_eq!(image.get_number_of_layers(), 1);

        Ok(())
    }

    #[test]
    fn test_get_layer_by_index() -> Result<(), ErrorTrace> {
        let image: QcowImage = get_image()?;

        let layer: QcowImageLayer = image.get_layer_by_index(0)?;

        match layer.read() {
            Ok(file) => assert_eq!(file.media_size, 4194304),
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to obtain read lock on QCOW layer",
                    error
                ));
            }
        };
        Ok(())
    }

    #[test]
    fn test_open() -> Result<(), ErrorTrace> {
        let mut image: QcowImage = QcowImage::new();

        let path_buf: PathBuf = PathBuf::from("../test_data/qcow");
        let file_resolver: FileResolverReference = open_os_file_resolver(&path_buf)?;
        let file_name: PathComponent = PathComponent::from("ext2.qcow2");
        image.open(&file_resolver, &file_name)?;

        Ok(())
    }
}
