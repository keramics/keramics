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

use std::collections::{HashMap, HashSet};
use std::io::SeekFrom;
use std::sync::{Arc, RwLock};

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_types::Uuid;

use crate::fake_file_resolver::FakeFileResolver;
use crate::file_resolver::FileResolverReference;
use crate::path_component::PathComponent;
use crate::xml::{XmlDocument, XmlElement};

use super::descriptor_extent::PdiDescriptorExtent;
use super::descriptor_image::PdiDescriptorImage;
use super::descriptor_snapshot::PdiDescriptorSnapshot;
use super::enums::{PdiDescriptorImageType, PdiExtentType};
use super::image_layer::PdiImageLayer;
use super::sparse_file_header::PdiSparseFileHeader;

/// Parallels Disk Image (PDI).
pub struct PdiImage {
    /// File resolver.
    file_resolver: FileResolverReference,

    /// Bytes per sector.
    pub bytes_per_sector: u16,

    /// Extents.
    extents: Vec<PdiDescriptorExtent>,

    /// Snapshots.
    snapshots: Vec<PdiDescriptorSnapshot>,

    /// Layers.
    layers: Vec<Arc<RwLock<PdiImageLayer>>>,

    /// Media size.
    pub media_size: u64,
}

impl PdiImage {
    /// Creates a new storage media image.
    pub fn new() -> Self {
        Self {
            file_resolver: FileResolverReference::new(Box::new(FakeFileResolver::new())),
            bytes_per_sector: 0,
            extents: Vec::new(),
            snapshots: Vec::new(),
            layers: Vec::new(),
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
    ) -> Result<Arc<RwLock<PdiImageLayer>>, ErrorTrace> {
        match self.layers.get(layer_index) {
            Some(image_layer) => Ok(image_layer.clone()),
            None => Err(keramics_core::error_trace_new!(format!(
                "No layer with index: {}",
                layer_index
            ))),
        }
    }

    /// Opens a storage media image.
    pub fn open(&mut self, file_resolver: &FileResolverReference) -> Result<(), ErrorTrace> {
        match self.read_disk_descriptor(&file_resolver, "DiskDescriptor.xml") {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read DiskDescriptor.xml");
                return Err(error);
            }
        }
        match self.read_extent_files(file_resolver) {
            Ok(()) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read extent files");
                return Err(error);
            }
        }
        self.file_resolver = file_resolver.clone();

        Ok(())
    }

    /// Reads a DiskDescriptor.xml file.
    fn read_disk_descriptor(
        &mut self,
        file_resolver: &FileResolverReference,
        file_name: &str,
    ) -> Result<(), ErrorTrace> {
        let path_components: [PathComponent; 1] = [PathComponent::from(file_name)];

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
        let data_stream_size: u64 = keramics_core::data_stream_get_size!(data_stream);

        if data_stream_size == 0 || data_stream_size > 65536 {
            return Err(keramics_core::error_trace_new!("Unsupported file size"));
        }
        let mut data: Vec<u8> = vec![0; data_stream_size as usize];

        keramics_core::data_stream_read_at_position!(data_stream, &mut data, SeekFrom::Start(0));

        keramics_core::debug_trace_data!("PdiImageXml", 0, &data, data_stream_size);

        let string: String = match String::from_utf8(data) {
            Ok(string) => string,
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to convert XML data into UTF-8 string",
                    error
                ));
            }
        };
        let mut xml_document: XmlDocument = XmlDocument::new();

        match xml_document.parse(string.as_str()) {
            Ok(_) => {}
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to parse XML document",
                    error
                ));
            }
        }
        match xml_document.root_element {
            Some(xml_element) => {
                if xml_element.name != "Parallels_disk_image" {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported XML document - unsupported root element: {}",
                        xml_element.name
                    )));
                }
                for sub_xml_element in xml_element.sub_elements.iter() {
                    match sub_xml_element.name.as_str() {
                        "Disk_Parameters" => match self.read_disk_parameters(sub_xml_element) {
                            Ok(_) => {}
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    "Unable to read disk parameters"
                                );
                                return Err(error);
                            }
                        },
                        "Snapshots" => match self.read_snapshots(sub_xml_element) {
                            Ok(_) => {}
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    "Unable to read snapshots"
                                );
                                return Err(error);
                            }
                        },
                        "StorageData" => match self.read_storage_data(sub_xml_element) {
                            Ok(_) => {}
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    "Unable to read storage data"
                                );
                                return Err(error);
                            }
                        },
                        _ => {}
                    }
                }
            }
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Unsupported XML document - missing root element"
                ));
            }
        }
        Ok(())
    }

    /// Reads disk parameters from DiskDescriptor.xml.
    fn read_disk_parameters(&mut self, xml_element: &XmlElement) -> Result<(), ErrorTrace> {
        let mut disk_size: u64 = 0;
        let mut logical_sector_size: u64 = 512;
        let mut physical_sector_size: u64 = 4096;

        for sub_xml_element in xml_element.sub_elements.iter() {
            match sub_xml_element.name.as_str() {
                "Cylinders" => match u64::from_str_radix(&sub_xml_element.value, 10) {
                    Ok(_) => {}
                    Err(error) => {
                        return Err(keramics_core::error_trace_new_with_error!(
                            "Unsupported Cylinders value",
                            error
                        ));
                    }
                },
                "Disk_size" => {
                    disk_size = match u64::from_str_radix(&sub_xml_element.value, 10) {
                        Ok(integer) => integer,
                        Err(error) => {
                            return Err(keramics_core::error_trace_new_with_error!(
                                "Unsupported Disk_size value",
                                error
                            ));
                        }
                    }
                }
                // TODO: add support for "Encryption"
                "Heads" => match u64::from_str_radix(&sub_xml_element.value, 10) {
                    Ok(_) => {}
                    Err(error) => {
                        return Err(keramics_core::error_trace_new_with_error!(
                            "Unsupported Heads value",
                            error
                        ));
                    }
                },
                // TODO: add support for "Miscellaneous"
                // TODO: add support for "Name"
                "LogicSectorSize" => {
                    logical_sector_size = match u64::from_str_radix(&sub_xml_element.value, 10) {
                        Ok(integer) => integer,
                        Err(error) => {
                            return Err(keramics_core::error_trace_new_with_error!(
                                "Unsupported LogicSectorSize value",
                                error
                            ));
                        }
                    }
                }
                "Padding" => match u64::from_str_radix(&sub_xml_element.value, 10) {
                    Ok(integer) => {
                        if integer != 0 {
                            return Err(keramics_core::error_trace_new!(format!(
                                "Unsupported padding value: {}",
                                integer
                            )));
                        }
                    }
                    Err(error) => {
                        return Err(keramics_core::error_trace_new_with_error!(
                            "Unsupported Padding value",
                            error
                        ));
                    }
                },
                "PhysicalSectorSize" => {
                    physical_sector_size = match u64::from_str_radix(&sub_xml_element.value, 10) {
                        Ok(integer) => integer,
                        Err(error) => {
                            return Err(keramics_core::error_trace_new_with_error!(
                                "Unsupported PhysicalSectorSize value",
                                error
                            ));
                        }
                    }
                }
                "Sectors" => match u64::from_str_radix(&sub_xml_element.value, 10) {
                    Ok(_) => {}
                    Err(error) => {
                        return Err(keramics_core::error_trace_new_with_error!(
                            "Unsupported Sectors value",
                            error
                        ));
                    }
                },
                "UID" => match Uuid::from_string(sub_xml_element.value.as_str()) {
                    Ok(_) => {}
                    Err(error) => {
                        return Err(keramics_core::error_trace_new_with_error!(
                            "Unsupported UID value",
                            error
                        ));
                    }
                },
                _ => {}
            }
        }
        if logical_sector_size != 512 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported logical sector size: {}",
                logical_sector_size
            )));
        }
        if physical_sector_size != 4096 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported physical sector size: {}",
                physical_sector_size
            )));
        }
        self.bytes_per_sector = 512;

        if disk_size > u64::MAX / (self.bytes_per_sector as u64) {
            return Err(keramics_core::error_trace_new!(
                "Unsupported disk size value out of bounds"
            ));
        }
        self.media_size = disk_size * (self.bytes_per_sector as u64);

        Ok(())
    }

    /// Reads the extent files.
    pub fn read_extent_files(
        &mut self,
        file_resolver: &FileResolverReference,
    ) -> Result<(), ErrorTrace> {
        let snapshots: HashSet<&Uuid> = self
            .snapshots
            .iter()
            .map(|descriptor_snapshot| &descriptor_snapshot.identifier)
            .collect();

        let mut last_end_sector: u64 = 0;

        for descriptor_extent in self.extents.iter() {
            if descriptor_extent.start_sector >= descriptor_extent.end_sector {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unsupported extent start sector: {} value exceeds end sector: {}",
                    descriptor_extent.start_sector, descriptor_extent.end_sector
                )));
            }
            let extent_number_of_sectors: u64 =
                descriptor_extent.end_sector - descriptor_extent.start_sector;

            if descriptor_extent.start_sector != last_end_sector {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unsupported extent start sector: {} value not aligned with last end sector: {}",
                    descriptor_extent.start_sector, last_end_sector
                )));
            }
            last_end_sector = descriptor_extent.end_sector;

            for descriptor_image in descriptor_extent.images.iter() {
                if !snapshots.contains(&descriptor_image.snapshot_identifier) {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Missing snapshot: {}",
                        descriptor_image.snapshot_identifier
                    )));
                }
                let path_components: [PathComponent; 1] =
                    [PathComponent::from(&descriptor_image.file)];

                let data_stream: DataStreamReference =
                    match file_resolver.get_data_stream(&path_components) {
                        Ok(Some(data_stream)) => data_stream,
                        Ok(None) => {
                            return Err(keramics_core::error_trace_new!(format!(
                                "Missing image data stream: {}",
                                descriptor_image.file
                            )));
                        }
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!("Unable to open image file: {}", descriptor_image.file,)
                            );
                            return Err(error);
                        }
                    };
                match &descriptor_image.image_type {
                    PdiDescriptorImageType::Compressed => {
                        let mut file_header: PdiSparseFileHeader = PdiSparseFileHeader::new();

                        match file_header.read_at_position(&data_stream, SeekFrom::Start(0)) {
                            Ok(_) => {}
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    "Unable to read sparse file header"
                                );
                                return Err(error);
                            }
                        }
                        if file_header.number_of_sectors != extent_number_of_sectors {
                            return Err(keramics_core::error_trace_new!(format!(
                                "Unsupported sparse file header number of sectors: {} value does not align with extent: {}",
                                file_header.number_of_sectors, extent_number_of_sectors
                            )));
                        }
                    }
                    PdiDescriptorImageType::Plain => {
                        let file_size: u64 = keramics_core::data_stream_get_size!(data_stream);

                        let extent_size: u64 =
                            extent_number_of_sectors * (self.bytes_per_sector as u64);

                        if file_size != extent_size {
                            return Err(keramics_core::error_trace_new!(format!(
                                "Unsupported file size: {} value does not align with extent: {}",
                                file_size, extent_size
                            )));
                        }
                    }
                    _ => {
                        return Err(keramics_core::error_trace_new!("Unsupported image type"));
                    }
                }
            }
        }
        // Note that the snapshots are not necessarily stored in-order.
        let mut layers: HashMap<&Uuid, PdiImageLayer> = HashMap::new();

        for descriptor_snapshot in self.snapshots.iter() {
            let mut image_layer: PdiImageLayer = PdiImageLayer::new(
                &descriptor_snapshot.identifier,
                descriptor_snapshot.parent_identifier.as_ref(),
                self.media_size,
            );
            for descriptor_extent in self.extents.iter() {
                for descriptor_image in descriptor_extent.images.iter() {
                    if descriptor_image.snapshot_identifier == descriptor_snapshot.identifier {
                        let offset: u64 =
                            descriptor_extent.start_sector * (self.bytes_per_sector as u64);
                        let size: u64 = (descriptor_extent.end_sector
                            - descriptor_extent.start_sector)
                            * (self.bytes_per_sector as u64);

                        let extent_type: PdiExtentType =
                            if descriptor_image.image_type == PdiDescriptorImageType::Compressed {
                                PdiExtentType::Sparse
                            } else {
                                PdiExtentType::Raw
                            };
                        image_layer.add_extent(
                            offset,
                            size,
                            descriptor_image.file.as_str(),
                            extent_type,
                        );
                    }
                }
            }
            layers.insert(&descriptor_snapshot.identifier, image_layer);
        }
        // Determine the order of the layers based on the number of ancestors.
        let mut layer_chains: Vec<(usize, &Uuid)> = Vec::new();

        for (layer_identifier, mut image_layer) in layers.iter() {
            let mut number_of_ancestors: usize = 0;
            let mut parent_identifier: Option<&Uuid> = image_layer.parent_identifier.as_ref();

            while let Some(identifier) = parent_identifier {
                match layers.get(identifier) {
                    Some(parent_image_layer) => {
                        number_of_ancestors += 1;
                        parent_identifier = parent_image_layer.parent_identifier.as_ref();
                    }
                    None => {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Missing layer: {}",
                            identifier
                        )));
                    }
                }
            }
            layer_chains.push((number_of_ancestors, layer_identifier));
        }
        layer_chains.sort();

        // Opens the layers starting from the base layer.
        let mut layer_indexes: HashMap<&Uuid, usize> = HashMap::new();

        for (_, layer_identifier) in layer_chains.iter() {
            let mut image_layer: PdiImageLayer = match layers.remove(layer_identifier) {
                Some(image_layer) => image_layer,
                None => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Missing layer: {}",
                        layer_identifier
                    )));
                }
            };
            if let Some(parent_identifier) = image_layer.parent_identifier.as_ref() {
                let parent_image_layer: &Arc<RwLock<PdiImageLayer>> =
                    match layer_indexes.get(parent_identifier) {
                        Some(parent_layer_index) => &self.layers[*parent_layer_index],
                        None => {
                            return Err(keramics_core::error_trace_new!(format!(
                                "Missing layer: {}",
                                parent_identifier
                            )));
                        }
                    };
                match image_layer.set_parent(parent_image_layer) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to set parent");
                        return Err(error);
                    }
                }
            }
            match image_layer.open(file_resolver) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to open layer: {}", layer_identifier)
                    );
                    return Err(error);
                }
            }
            layer_indexes.insert(layer_identifier, self.layers.len());

            self.layers.push(Arc::new(RwLock::new(image_layer)));
        }
        Ok(())
    }

    /// Reads an image from DiskDescriptor.xml.
    fn read_image(&self, xml_element: &XmlElement) -> Result<PdiDescriptorImage, ErrorTrace> {
        let mut file: String = String::new();
        let mut image_type: PdiDescriptorImageType = PdiDescriptorImageType::NotSet;
        let mut snapshot_identifier: Uuid = Uuid::new();

        for sub_xml_element in xml_element.sub_elements.iter() {
            match sub_xml_element.name.as_str() {
                "File" => file = sub_xml_element.value.clone(),
                "GUID" => {
                    snapshot_identifier = match Uuid::from_string(sub_xml_element.value.as_str()) {
                        Ok(uuid) => uuid,
                        Err(error) => {
                            return Err(keramics_core::error_trace_new_with_error!(
                                "Unsupported GUID value",
                                error
                            ));
                        }
                    }
                }
                "Type" => {
                    image_type = match sub_xml_element.value.as_str() {
                        "Compressed" => PdiDescriptorImageType::Compressed,
                        "Plain" => PdiDescriptorImageType::Plain,
                        _ => {
                            return Err(keramics_core::error_trace_new!(format!(
                                "Unsupported Type value: {}",
                                sub_xml_element.value
                            )));
                        }
                    }
                }
                _ => {}
            }
        }
        if file.is_empty() {
            return Err(keramics_core::error_trace_new!("Missing File value"));
        }
        if image_type == PdiDescriptorImageType::NotSet {
            return Err(keramics_core::error_trace_new!("Missing Type value"));
        }
        if snapshot_identifier.is_nil() {
            return Err(keramics_core::error_trace_new!(
                "Missing or unsupported GUID value"
            ));
        }
        Ok(PdiDescriptorImage::new(
            file,
            image_type,
            snapshot_identifier,
        ))
    }

    /// Reads a snapshot from DiskDescriptor.xml.
    fn read_snapshot(&self, xml_element: &XmlElement) -> Result<PdiDescriptorSnapshot, ErrorTrace> {
        let mut identifier: Uuid = Uuid::new();
        let mut parent_identifier: Option<Uuid> = None;

        for sub_xml_element in xml_element.sub_elements.iter() {
            match sub_xml_element.name.as_str() {
                "GUID" => {
                    identifier = match Uuid::from_string(sub_xml_element.value.as_str()) {
                        Ok(uuid) => uuid,
                        Err(error) => {
                            return Err(keramics_core::error_trace_new_with_error!(
                                "Unsupported GUID value",
                                error
                            ));
                        }
                    }
                }
                "ParentGUID" => {
                    parent_identifier = match Uuid::from_string(sub_xml_element.value.as_str()) {
                        Ok(uuid) => {
                            if uuid.is_nil() {
                                None
                            } else {
                                Some(uuid)
                            }
                        }
                        Err(error) => {
                            return Err(keramics_core::error_trace_new_with_error!(
                                "Unsupported ParentGUID value",
                                error
                            ));
                        }
                    }
                }
                _ => {}
            }
        }
        if identifier.is_nil() {
            return Err(keramics_core::error_trace_new!(
                "Missing or unsupported GUID value"
            ));
        }
        Ok(PdiDescriptorSnapshot::new(identifier, parent_identifier))
    }

    /// Reads snapshots from DiskDescriptor.xml.
    fn read_snapshots(&mut self, xml_element: &XmlElement) -> Result<(), ErrorTrace> {
        for sub_xml_element in xml_element.sub_elements.iter() {
            match sub_xml_element.name.as_str() {
                "Shot" => match self.read_snapshot(sub_xml_element) {
                    Ok(descriptor_snapshot) => self.snapshots.push(descriptor_snapshot),
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to read snapshot");
                        return Err(error);
                    }
                },
                _ => {}
            }
        }
        Ok(())
    }

    /// Reads a storage from DiskDescriptor.xml.
    fn read_storage(&self, xml_element: &XmlElement) -> Result<PdiDescriptorExtent, ErrorTrace> {
        let mut block_size: u64 = 0;
        let mut end_sector: u64 = 0;
        let mut images: Vec<PdiDescriptorImage> = Vec::new();
        let mut start_sector: u64 = 0;

        for sub_xml_element in xml_element.sub_elements.iter() {
            match sub_xml_element.name.as_str() {
                "Blocksize" => {
                    block_size = match u64::from_str_radix(&sub_xml_element.value, 10) {
                        Ok(integer) => integer,
                        Err(error) => {
                            return Err(keramics_core::error_trace_new_with_error!(
                                "Unsupported Blocksize value",
                                error
                            ));
                        }
                    }
                }
                "End" => {
                    end_sector = match u64::from_str_radix(&sub_xml_element.value, 10) {
                        Ok(integer) => integer,
                        Err(error) => {
                            return Err(keramics_core::error_trace_new_with_error!(
                                "Unsupported End value",
                                error
                            ));
                        }
                    }
                }
                "Image" => match self.read_image(sub_xml_element) {
                    Ok(descriptor_image) => images.push(descriptor_image),
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to read image");
                        return Err(error);
                    }
                },
                "Start" => {
                    start_sector = match u64::from_str_radix(&sub_xml_element.value, 10) {
                        Ok(integer) => integer,
                        Err(error) => {
                            return Err(keramics_core::error_trace_new_with_error!(
                                "Unsupported Start value",
                                error
                            ));
                        }
                    }
                }
                _ => {}
            }
        }
        if block_size != 2048 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported block size: {}",
                block_size
            )));
        }
        Ok(PdiDescriptorExtent::new(start_sector, end_sector, images))
    }

    /// Reads storage data from DiskDescriptor.xml.
    fn read_storage_data(&mut self, xml_element: &XmlElement) -> Result<(), ErrorTrace> {
        for sub_xml_element in xml_element.sub_elements.iter() {
            match sub_xml_element.name.as_str() {
                "Storage" => match self.read_storage(sub_xml_element) {
                    Ok(descriptor_extent) => self.extents.push(descriptor_extent),
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to read storage");
                        return Err(error);
                    }
                },
                _ => {}
            }
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

    fn get_image() -> Result<PdiImage, ErrorTrace> {
        let mut image: PdiImage = PdiImage::new();

        let path_string: String = get_test_data_path("pdi/hfsplus.hdd");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let file_resolver: FileResolverReference = open_os_file_resolver(&path_buf)?;
        image.open(&file_resolver)?;

        Ok(image)
    }

    #[test]
    fn test_get_number_of_layers() -> Result<(), ErrorTrace> {
        let image: PdiImage = get_image()?;

        assert_eq!(image.get_number_of_layers(), 1);

        Ok(())
    }

    #[test]
    fn test_get_layer_by_index() -> Result<(), ErrorTrace> {
        let image: PdiImage = get_image()?;

        let layer: Arc<RwLock<PdiImageLayer>> = image.get_layer_by_index(0)?;

        match layer.read() {
            Ok(image_layer) => assert_eq!(image_layer.media_size, 33554432),
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to obtain read lock on PDI layer",
                    error
                ));
            }
        }
        Ok(())
    }

    #[test]
    fn test_open() -> Result<(), ErrorTrace> {
        let mut image: PdiImage = PdiImage::new();

        let path_string: String = get_test_data_path("pdi/hfsplus.hdd");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let file_resolver: FileResolverReference = open_os_file_resolver(&path_buf)?;
        image.open(&file_resolver)?;

        assert_eq!(image.media_size, 33554432);

        Ok(())
    }

    // TODO: add tests for read_disk_descriptor
    // TODO: add tests for read_disk_parameters
    // TODO: add tests for read_extent_files
    // TODO: add tests for read_image
    // TODO: add tests for read_snapshot
    // TODO: add tests for read_snapshots
    // TODO: add tests for read_storage
    // TODO: add tests for read_storage_data
}
