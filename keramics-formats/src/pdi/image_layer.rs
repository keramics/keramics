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
use keramics_types::Uuid;

use crate::file_resolver::FileResolverReference;

use super::block_reader::PdiBlockReader;
use super::block_stream::PdiBlockStream;
use super::enums::PdiExtentType;
use super::image_extent::PdiImageExtent;

/// Parallels Disk Image (PDI) layer.
pub struct PdiImageLayer {
    /// File resolver.
    file_resolver: FileResolverReference,

    /// Identifier.
    pub(super) identifier: Uuid,

    /// Extents.
    extents: Vec<PdiImageExtent>,

    /// Parent identifier.
    pub(super) parent_identifier: Option<Uuid>,

    /// Parent layer.
    parent_layer: Option<Arc<PdiImageLayer>>,

    /// Media size.
    pub(super) media_size: u64,
}

impl PdiImageLayer {
    /// Creates a new image layer.
    pub(super) fn new(
        file_resolver: &FileResolverReference,
        identifier: &Uuid,
        parent_identifier: Option<&Uuid>,
        media_size: u64,
    ) -> Self {
        Self {
            file_resolver: file_resolver.clone(),
            identifier: identifier.clone(),
            parent_identifier: parent_identifier.cloned(),
            parent_layer: None,
            extents: Vec::new(),
            media_size,
        }
    }

    /// Adds an extent.
    pub(super) fn add_extent(
        &mut self,
        offset: u64,
        size: u64,
        file_name: &str,
        extent_type: PdiExtentType,
    ) {
        let extent: PdiImageExtent = PdiImageExtent::new(offset, size, file_name, extent_type);
        self.extents.push(extent);
    }

    /// Retrieves a data stream.
    pub fn get_data_stream(&self) -> DataStreamReference {
        let parent_data_stream: Option<DataStreamReference> = match &self.parent_layer {
            Some(parent_layer) => Some(parent_layer.get_data_stream()),
            None => None,
        };
        Arc::new(RwLock::new(PdiBlockStream::new(PdiBlockReader::new(
            &self.file_resolver,
            &self.extents,
            parent_data_stream,
            self.media_size,
        ))))
    }

    /// Retrieves the identifier.
    pub fn get_identifier(&self) -> &Uuid {
        &self.identifier
    }

    /// Retrieves the media size.
    pub fn get_media_size(&self) -> u64 {
        self.media_size
    }

    /// Sets the parent layer.
    pub fn set_parent(&mut self, parent_layer: &Arc<PdiImageLayer>) -> Result<(), ErrorTrace> {
        let parent_identifier: &Uuid = match &self.parent_identifier {
            Some(parent_identifier) => parent_identifier,
            None => {
                return Err(keramics_core::error_trace_new!("Missing parent identifier"));
            }
        };
        if parent_identifier != &parent_layer.identifier {
            return Err(keramics_core::error_trace_new!(format!(
                "Parent identifier: {} does not match identifier of parent layer: {}",
                parent_identifier, parent_layer.identifier,
            )));
        }
        self.parent_layer = Some(parent_layer.clone());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use crate::os_file_resolver::open_os_file_resolver;
    use crate::tests::get_test_data_path;

    fn get_image_layer() -> Result<PdiImageLayer, ErrorTrace> {
        let path_string: String = get_test_data_path("pdi/hfsplus.hdd");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let file_resolver: FileResolverReference = open_os_file_resolver(&path_buf)?;

        let identifier: Uuid = Uuid::from_string("{5fbaabe3-6958-40ff-92a7-860e329aab41}")?;

        let mut image_layer: PdiImageLayer =
            PdiImageLayer::new(&file_resolver, &identifier, None, 33554432);
        image_layer.add_extent(
            0,
            33554432,
            "hfsplus.hdd.0.{5fbaabe3-6958-40ff-92a7-860e329aab41}.hds",
            PdiExtentType::Sparse,
        );
        Ok(image_layer)
    }

    // TODO: add test for add_extent
    // TODO: add test for get_data_stream
    // TODO: add test for get_extent_file

    #[test]
    fn test_get_identifier() -> Result<(), ErrorTrace> {
        let image_layer: PdiImageLayer = get_image_layer()?;

        let identifier: &Uuid = image_layer.get_identifier();
        assert_eq!(
            identifier.to_string(),
            "5fbaabe3-6958-40ff-92a7-860e329aab41"
        );
        Ok(())
    }

    #[test]
    fn test_get_media_size() -> Result<(), ErrorTrace> {
        let image_layer: PdiImageLayer = get_image_layer()?;

        let media_size: u64 = image_layer.get_media_size();
        assert_eq!(media_size, 33554432);

        Ok(())
    }

    // TODO: add test for set_parent
}
