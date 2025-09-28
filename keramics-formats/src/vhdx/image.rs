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

use super::file::VhdxFile;

pub type VhdxImageLayer = Arc<RwLock<VhdxFile>>;

/// Virtual Hard Disk version 2 (VHDX) storage media image.
pub struct VhdxImage {
    /// Files.
    files: Vec<Arc<RwLock<VhdxFile>>>,
}

impl VhdxImage {
    /// Creates a new storage media image.
    pub fn new() -> Self {
        Self { files: Vec::new() }
    }

    /// Retrieves the number of layers.
    pub fn get_number_of_layers(&self) -> usize {
        self.files.len()
    }

    /// Retrieves a layer by index.
    pub fn get_layer_by_index(&self, layer_index: usize) -> io::Result<VhdxImageLayer> {
        match self.files.get(layer_index) {
            Some(file) => Ok(file.clone()),
            None => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("No layer with index: {}", layer_index),
            )),
        }
    }

    // TODO: add get_layer_index_by_identifier

    /// Opens a storage media image.
    pub fn open(
        &mut self,
        file_resolver: &FileResolverReference,
        file_name: &str,
    ) -> io::Result<()> {
        let mut files: Vec<VhdxFile> = Vec::new();

        let mut file: VhdxFile = VhdxFile::new();

        match file_resolver.get_data_stream(&mut vec![file_name])? {
            Some(data_stream) => file.read_data_stream(&data_stream)?,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("No such file: {}", file_name),
                ));
            }
        };
        while file.parent_identifier.is_some() {
            let parent_file_name: String = match file.get_parent_file_name() {
                Some(file_name) => file_name.to_string(),
                None => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Missing parent file_name",
                    ));
                }
            };
            let mut parent_file: VhdxFile = VhdxFile::new();

            match file_resolver.get_data_stream(&mut vec![parent_file_name.as_str()])? {
                Some(data_stream) => parent_file.read_data_stream(&data_stream)?,
                None => {
                    return Err(io::Error::new(
                        io::ErrorKind::NotFound,
                        format!("Missing parent file: {}", parent_file_name),
                    ));
                }
            };
            files.push(file);

            file = parent_file;
        }
        files.push(file);

        let mut file_index: usize = 0;
        while let Some(mut file) = files.pop() {
            if file_index > 0 {
                file.set_parent(&mut self.files[file_index - 1])?;
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

    fn get_image() -> io::Result<VhdxImage> {
        let mut image: VhdxImage = VhdxImage::new();

        let file_resolver: FileResolverReference = open_os_file_resolver("../test_data/vhdx")?;
        image.open(&file_resolver, "ntfs-differential.vhdx")?;

        Ok(image)
    }

    #[test]
    fn test_get_layer_by_index() -> io::Result<()> {
        let image: VhdxImage = get_image()?;

        let layer: VhdxImageLayer = image.get_layer_by_index(0)?;

        match layer.read() {
            Ok(file) => {
                assert_eq!(file.media_size, 4194304);
                assert_eq!(
                    file.identifier.to_string(),
                    "7584f8fb-36d3-4091-afb5-b1afe587bfa8"
                );
            }
            Err(error) => return Err(keramics_core::error_to_io_error!(error)),
        };
        Ok(())
    }

    #[test]
    fn test_open() -> io::Result<()> {
        let mut image: VhdxImage = VhdxImage::new();

        let file_resolver: FileResolverReference = open_os_file_resolver("../test_data/vhdx")?;
        image.open(&file_resolver, "ntfs-differential.vhdx")?;

        Ok(())
    }
}
