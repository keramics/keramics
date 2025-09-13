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

use std::io;
use std::sync::{Arc, RwLock};

use keramics_core::FileResolverReference;

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
    pub fn get_layer_by_index(&self, layer_index: usize) -> io::Result<QcowImageLayer> {
        match self.files.get(layer_index) {
            Some(file) => Ok(file.clone()),
            None => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("No layer with index: {}", layer_index),
            )),
        }
    }

    /// Opens a storage media image.
    pub fn open(
        &mut self,
        file_resolver: &FileResolverReference,
        filename: &str,
    ) -> io::Result<()> {
        let mut files: Vec<QcowFile> = Vec::new();

        let mut file: QcowFile = QcowFile::new();

        match file_resolver.get_data_stream(&mut vec![filename])? {
            Some(data_stream) => file.read_data_stream(&data_stream)?,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("No such file: {}", filename),
                ))
            }
        };
        while let Some(file_name) = file.get_backing_file_name() {
            let mut backing_file: QcowFile = QcowFile::new();

            let backing_file_name: String = file_name.to_string();

            match file_resolver.get_data_stream(&mut vec![backing_file_name.as_str()])? {
                Some(data_stream) => backing_file.read_data_stream(&data_stream)?,
                None => {
                    return Err(io::Error::new(
                        io::ErrorKind::NotFound,
                        format!("Missing backing file: {}", backing_file_name),
                    ))
                }
            };
            files.push(file);

            file = backing_file;
        }
        files.push(file);

        let mut file_index: usize = 0;
        while let Some(mut file) = files.pop() {
            if file_index > 0 {
                file.set_backing_file(&mut self.files[file_index - 1])?;
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

    use keramics_core::open_os_file_resolver;

    fn get_image() -> io::Result<QcowImage> {
        let mut image: QcowImage = QcowImage::new();

        let file_resolver: FileResolverReference = open_os_file_resolver("../test_data/qcow")?;
        image.open(&file_resolver, "ext2.qcow2")?;

        Ok(image)
    }

    #[test]
    fn test_get_number_of_layers() -> io::Result<()> {
        let image: QcowImage = get_image()?;

        assert_eq!(image.get_number_of_layers(), 1);

        Ok(())
    }

    #[test]
    fn test_get_layer_by_index() -> io::Result<()> {
        let image: QcowImage = get_image()?;

        let layer: QcowImageLayer = image.get_layer_by_index(0)?;

        match layer.read() {
            Ok(file) => assert_eq!(file.media_size, 4194304),
            Err(error) => return Err(keramics_core::error_to_io_error!(error)),
        };
        Ok(())
    }

    #[test]
    fn test_open() -> io::Result<()> {
        let mut image: QcowImage = QcowImage::new();

        let file_resolver: FileResolverReference = open_os_file_resolver("../test_data/qcow")?;
        image.open(&file_resolver, "ext2.qcow2")?;

        Ok(())
    }
}
