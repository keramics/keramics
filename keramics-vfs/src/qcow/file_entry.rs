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

use keramics_core::DataStreamReference;
use keramics_formats::qcow::{QcowFile, QcowImage};

use crate::image::{VfsImageFileEntry, VfsImageIdentifier};
use crate::traits::VfsImageLayer;

/// QEMU Copy-On-Write (QCOW) storage media image file entry.
pub type QcowFileEntry = VfsImageFileEntry<QcowImage, QcowFile>;

impl VfsImageLayer for QcowFile {
    /// Name prefix.
    const NAME_PREFIX: &'static str = "qcow";

    /// Retrieves the default data stream.
    fn get_data_stream(&self) -> Option<DataStreamReference> {
        QcowFile::get_data_stream(self)
    }

    /// Retrieves the identifier.
    fn get_identifier(&self) -> Option<VfsImageIdentifier> {
        None
    }

    /// Retrieves the media size.
    fn get_media_size(&self) -> u64 {
        QcowFile::get_media_size(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;
    use std::sync::Arc;

    use keramics_core::ErrorTrace;
    use keramics_formats::{FileResolverReference, PathComponent, open_os_file_resolver};

    use crate::enums::VfsFileType;
    use crate::tests::get_test_data_path;

    fn get_image() -> Result<Arc<QcowImage>, ErrorTrace> {
        let mut image: QcowImage = QcowImage::new();

        let path_string: String = get_test_data_path("qcow");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let file_resolver: FileResolverReference = open_os_file_resolver(&path_buf)?;
        let file_name: PathComponent = PathComponent::from("ext2.qcow2");
        image.open(&file_resolver, &file_name)?;

        Ok(Arc::new(image))
    }

    fn get_layer_file_entry(image: &Arc<QcowImage>) -> Result<QcowFileEntry, ErrorTrace> {
        let image_layer: Arc<QcowFile> = image.get_layer_by_index(0)?;

        Ok(QcowFileEntry::Layer {
            name_index: 0,
            layer: image_layer,
        })
    }

    fn get_root_file_entry(image: &Arc<QcowImage>) -> QcowFileEntry {
        QcowFileEntry::Root {
            image: image.clone(),
        }
    }

    // TODO: add tests for get_data_stream

    #[test]
    fn test_get_file_type() -> Result<(), ErrorTrace> {
        let test_image: Arc<QcowImage> = get_image()?;

        let file_entry: QcowFileEntry = get_root_file_entry(&test_image);

        let file_type: VfsFileType = file_entry.get_file_type();
        assert_eq!(file_type, VfsFileType::Directory);

        let file_entry: QcowFileEntry = get_layer_file_entry(&test_image)?;

        let file_type: VfsFileType = file_entry.get_file_type();
        assert_eq!(file_type, VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_identifier() -> Result<(), ErrorTrace> {
        let test_image: Arc<QcowImage> = get_image()?;

        let file_entry: QcowFileEntry = get_root_file_entry(&test_image);
        let result: Option<VfsImageIdentifier> = file_entry.get_identifier();
        assert!(result.is_none());

        let file_entry: QcowFileEntry = get_layer_file_entry(&test_image)?;
        let result: Option<VfsImageIdentifier> = file_entry.get_identifier();
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_name() -> Result<(), ErrorTrace> {
        let test_image: Arc<QcowImage> = get_image()?;

        let file_entry: QcowFileEntry = get_root_file_entry(&test_image);

        let name: PathComponent = file_entry.get_name();
        assert_eq!(name, PathComponent::Root);

        let file_entry: QcowFileEntry = get_layer_file_entry(&test_image)?;

        let name: PathComponent = file_entry.get_name();
        assert_eq!(name, PathComponent::from("qcow1"));

        Ok(())
    }

    #[test]
    fn test_get_layer_number() -> Result<(), ErrorTrace> {
        let test_image: Arc<QcowImage> = get_image()?;

        let file_entry: QcowFileEntry = get_root_file_entry(&test_image);

        let layer_number: Option<usize> = file_entry.get_layer_number();
        assert_eq!(layer_number, None);

        let file_entry: QcowFileEntry = get_layer_file_entry(&test_image)?;

        let layer_number: Option<usize> = file_entry.get_layer_number();
        assert_eq!(layer_number, Some(1));

        Ok(())
    }

    #[test]
    fn test_get_size() -> Result<(), ErrorTrace> {
        let test_image: Arc<QcowImage> = get_image()?;

        let file_entry: QcowFileEntry = get_root_file_entry(&test_image);

        let size: u64 = file_entry.get_size();
        assert_eq!(size, 0);

        let file_entry: QcowFileEntry = get_layer_file_entry(&test_image)?;

        let size: u64 = file_entry.get_size();
        assert_eq!(size, 4194304);

        Ok(())
    }

    #[test]
    fn test_get_number_of_sub_file_entries() -> Result<(), ErrorTrace> {
        let test_image: Arc<QcowImage> = get_image()?;

        let file_entry: QcowFileEntry = get_root_file_entry(&test_image);

        let number_of_sub_file_entries: usize = file_entry.get_number_of_sub_file_entries();
        assert_eq!(number_of_sub_file_entries, 1);

        let file_entry: QcowFileEntry = get_layer_file_entry(&test_image)?;

        let number_of_sub_file_entries: usize = file_entry.get_number_of_sub_file_entries();
        assert_eq!(number_of_sub_file_entries, 0);

        Ok(())
    }

    #[test]
    fn test_get_sub_file_entry_by_index() -> Result<(), ErrorTrace> {
        let test_image: Arc<QcowImage> = get_image()?;

        let file_entry: QcowFileEntry = get_root_file_entry(&test_image);

        let sub_file_entry: QcowFileEntry = file_entry.get_sub_file_entry_by_index(0)?;

        let name: PathComponent = sub_file_entry.get_name();
        assert_eq!(name, PathComponent::from("qcow1"));

        let result: Result<QcowFileEntry, ErrorTrace> = file_entry.get_sub_file_entry_by_index(99);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_is_root_file_entry() -> Result<(), ErrorTrace> {
        let test_image: Arc<QcowImage> = get_image()?;

        let file_entry: QcowFileEntry = get_root_file_entry(&test_image);

        assert_eq!(file_entry.is_root_file_entry(), true);

        let file_entry: QcowFileEntry = get_layer_file_entry(&test_image)?;

        assert_eq!(file_entry.is_root_file_entry(), false);

        Ok(())
    }
}
