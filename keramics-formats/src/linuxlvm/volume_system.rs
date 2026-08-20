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

use std::cmp::Ordering;
use std::collections::HashMap;
use std::io::SeekFrom;

use keramics_core::{DataStreamReference, ErrorTrace};

use crate::fake_file_resolver::FakeFileResolver;
use crate::file_resolver::FileResolverReference;
use crate::path_component::PathComponent;

use super::data_area_descriptor::LinuxLvmDataAreaDescriptor;
use super::data_file_descriptor::LinuxLvmDataFileDescriptor;
use super::extent::{LinuxLvmExtent, LinuxLvmExtentValues};
use super::logical_volume::LinuxLvmLogicalVolume;
use super::metadata::LinuxLvmMetadata;
use super::metadata_area_header::LinuxLvmMetadataAreaHeader;
use super::physical_volume::LinuxLvmPhysicalVolume;
use super::physical_volume_label::LinuxLvmPhysicalVolumeLabel;
use super::raw_location_descriptor::LinuxLvmRawLocationDescriptor;
use super::volume::LinuxLvmVolume;
use super::volume_group::LinuxLvmVolumeGroup;

/// Linux Logical Volume Manager (LVM) volume system.
pub struct LinuxLvmVolumeSystem {
    /// File resolver.
    file_resolver: FileResolverReference,

    /// Bytes per sector.
    bytes_per_sector: u16,

    /// Data file descriptors.
    data_file_descriptors: Vec<LinuxLvmDataFileDescriptor>,

    /// Volume group.
    volume_group: Option<LinuxLvmVolumeGroup>,
}

impl LinuxLvmVolumeSystem {
    /// Creates a new volume system.
    pub fn new() -> Self {
        Self {
            file_resolver: FileResolverReference::new(Box::new(FakeFileResolver::new())),
            bytes_per_sector: 0,
            data_file_descriptors: Vec::new(),
            volume_group: None,
        }
    }

    /// Retrieves the bytes per sector.
    pub fn get_bytes_per_sector(&self) -> u16 {
        self.bytes_per_sector
    }

    /// Retrieves the (volume group) identifier.
    pub fn get_identifier(&self) -> Option<&str> {
        match &self.volume_group {
            Some(volume_group) => Some(volume_group.identifier.as_str()),
            None => None,
        }
    }

    /// Retrieves the (volume group) name.
    pub fn get_name(&self) -> Option<&str> {
        match &self.volume_group {
            Some(volume_group) => Some(volume_group.name.as_str()),
            None => None,
        }
    }

    /// Retrieves the number of physical volumes.
    pub fn get_number_of_physical_volumes(&self) -> usize {
        match &self.volume_group {
            Some(volume_group) => volume_group.physical_volumes.len(),
            None => 0,
        }
    }

    /// Retrieves a physical volume by index.
    pub fn get_physical_volume_by_index(
        &self,
        volume_index: usize,
    ) -> Option<&LinuxLvmPhysicalVolume> {
        match &self.volume_group {
            Some(volume_group) => volume_group.physical_volumes.get(volume_index),
            None => None,
        }
    }

    /// Retrieves the number of (logical) volumes.
    pub fn get_number_of_volumes(&self) -> usize {
        match &self.volume_group {
            Some(volume_group) => volume_group.logical_volumes.len(),
            None => 0,
        }
    }

    /// Retrieves a volume by index.
    pub fn get_volume_by_index(&self, volume_index: usize) -> Result<LinuxLvmVolume, ErrorTrace> {
        let volume_group: &LinuxLvmVolumeGroup = match &self.volume_group {
            Some(volume_group) => volume_group,
            None => {
                return Err(keramics_core::error_trace_new!("Missing volume group"));
            }
        };
        let logical_volume: &LinuxLvmLogicalVolume =
            match volume_group.logical_volumes.get(volume_index) {
                Some(logical_volume) => logical_volume,
                None => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "No logical volume with index: {}",
                        volume_index
                    )));
                }
            };
        let mut volume: LinuxLvmVolume = LinuxLvmVolume::new(&self.file_resolver);

        volume.open(&self.data_file_descriptors, volume_index, logical_volume);

        Ok(volume)
    }

    /// Opens a storage media image.
    pub fn open(
        &mut self,
        file_resolver: &FileResolverReference,
        data_files: &[LinuxLvmDataFileDescriptor],
    ) -> Result<(), ErrorTrace> {
        let mut physical_volume_labels: HashMap<String, LinuxLvmPhysicalVolumeLabel> =
            HashMap::new();

        for data_file_descriptor in data_files.iter() {
            let path_components: [PathComponent; 1] = [data_file_descriptor.file_name.clone()];

            let data_stream: DataStreamReference =
                match file_resolver.get_data_stream(&path_components) {
                    Ok(Some(data_stream)) => data_stream,
                    Ok(None) => {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Missing data stream: {}",
                            data_file_descriptor.file_name
                        )));
                    }
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!("Unable to open file: {}", data_file_descriptor.file_name)
                        );
                        return Err(error);
                    }
                };
            let volume_label_offset: u64 = data_file_descriptor.start_offset + 512;

            let mut physical_volume_label: LinuxLvmPhysicalVolumeLabel =
                LinuxLvmPhysicalVolumeLabel::new();

            match physical_volume_label
                .read_at_position(&data_stream, SeekFrom::Start(volume_label_offset))
            {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to read physical volume label at offset: {} (0x{:08x})",
                            volume_label_offset, volume_label_offset
                        )
                    );
                    return Err(error);
                }
            }
            let data_area_descriptor: &LinuxLvmDataAreaDescriptor =
                match physical_volume_label.metadata_area_descriptors.get(0) {
                    Some(data_area_descriptor) => data_area_descriptor,
                    None => {
                        return Err(keramics_core::error_trace_new!(
                            "Missing metadata area descriptors"
                        ));
                    }
                };
            let volume_group: LinuxLvmVolumeGroup =
                match self.read_volume_group(&data_stream, data_area_descriptor) {
                    Ok(volume_group) => volume_group,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!(
                                "Unable to read volume group from file: {}",
                                data_file_descriptor.file_name
                            )
                        );
                        return Err(error);
                    }
                };
            physical_volume_labels.insert(
                physical_volume_label.identifier.to_string(),
                physical_volume_label,
            );
            // TODO: make sure the descriptors are in order of physical volume index
            self.data_file_descriptors
                .push(data_file_descriptor.clone());

            if self.volume_group.is_none() {
                self.volume_group = Some(volume_group);
            }
        }
        self.bytes_per_sector = 512;

        let mut physical_volumes: HashMap<String, (usize, String)> = HashMap::new();

        if let Some(volume_group) = &mut self.volume_group {
            let number_of_physical_volumes: usize = volume_group.physical_volumes.len();

            if number_of_physical_volumes == 0 {
                return Err(keramics_core::error_trace_new!("Missing physical volumes",));
            }
            // TODO: add support for multiple physical volumes.
            if volume_group.physical_volumes.len() != 1 {
                return Err(keramics_core::error_trace_new!(
                    "Unsupported number of physical volumes",
                ));
            }
            for physical_volume in volume_group.physical_volumes.iter() {
                let identifier: String = physical_volume.identifier.to_string();
                let lookup_identifier: String = identifier.replace("-", "");

                if !physical_volume_labels.contains_key(&lookup_identifier) {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Missing physical volume: {}",
                        identifier
                    )));
                }
                physical_volumes.insert(
                    physical_volume.name.clone(),
                    (physical_volume.index, lookup_identifier),
                );
            }
            let extent_size: u64 =
                (volume_group.extent_size as u64) * (self.bytes_per_sector as u64);

            for logical_volume in volume_group.logical_volumes.iter_mut() {
                let mut logical_volume_size: u64 = 0;

                for (segment_index, segment) in logical_volume.segments.iter().enumerate() {
                    let segment_offset: u64 = (segment.start_extent as u64) * extent_size;
                    let segment_size: u64 = (segment.number_of_extents as u64) * extent_size;

                    match segment.segment_type.as_str() {
                        "striped" => {
                            let number_of_stripes: usize = segment.stripes.len();

                            if number_of_stripes == 0 {
                                return Err(keramics_core::error_trace_new!(format!(
                                    "Unsupported segment: {} - missing stripes",
                                    segment_index
                                )));
                            }
                            // TODO: add support for multiple stripes.
                            if number_of_stripes != 1 {
                                return Err(keramics_core::error_trace_new!(
                                    "Unsupported segment: {} - unsupported number of stripes",
                                ));
                            }
                            let stripe_offset: u64 = segment_offset;
                            let stripe_size: u64 = segment_size;

                            for (stripe_index, stripe) in segment.stripes.iter().enumerate() {
                                let (physical_volume_index, physical_volume_identifier) =
                                    match physical_volumes.get(&stripe.physical_volume_name) {
                                        Some(result) => result,
                                        None => {
                                            return Err(keramics_core::error_trace_new!(format!(
                                                "Invalid segment: {} - invalid stripe: {} - missing physical volume: {}",
                                                segment_index,
                                                stripe_index,
                                                stripe.physical_volume_name
                                            )));
                                        }
                                    };
                                let physical_volume_label: &LinuxLvmPhysicalVolumeLabel =
                                    match physical_volume_labels.get(physical_volume_identifier) {
                                        Some(physical_volume_label) => physical_volume_label,
                                        None => {
                                            return Err(keramics_core::error_trace_new!(format!(
                                                "Missing physical volume label: {}",
                                                physical_volume_identifier
                                            )));
                                        }
                                    };
                                let data_area_offset: u64 =
                                    (stripe.start_extent as u64) * extent_size;

                                let data_area_decriptor_index: usize = match physical_volume_label
                                    .data_area_descriptors
                                    .binary_search_by(|data_area_descriptor| {
                                        let area_end_offset: u64 = data_area_descriptor
                                            .logical_offset
                                            + data_area_descriptor.size;

                                        if data_area_offset >= area_end_offset {
                                            Ordering::Less
                                        } else if data_area_offset
                                            < data_area_descriptor.logical_offset
                                        {
                                            Ordering::Greater
                                        } else {
                                            Ordering::Equal
                                        }
                                    }) {
                                    Ok(data_area_decriptor_index) => data_area_decriptor_index,
                                    Err(_) => {
                                        return Err(keramics_core::error_trace_new!(format!(
                                            "Missing data area descriptor for offset: {} (0x{:08x})",
                                            data_area_offset, data_area_offset
                                        )));
                                    }
                                };
                                let data_area_descriptor: &LinuxLvmDataAreaDescriptor =
                                    match physical_volume_label
                                        .data_area_descriptors
                                        .get(data_area_decriptor_index)
                                    {
                                        Some(data_area_descriptor) => data_area_descriptor,
                                        None => {
                                            return Err(keramics_core::error_trace_new!(format!(
                                                "Missing data area descriptor: {}",
                                                data_area_decriptor_index
                                            )));
                                        }
                                    };
                                let data_area_physical_offset: u64 = data_area_descriptor
                                    .physical_offset
                                    + (data_area_offset - data_area_descriptor.logical_offset);

                                let extent: LinuxLvmExtent = LinuxLvmExtent {
                                    logical_offset: stripe_offset,
                                    size: stripe_size,
                                    values: LinuxLvmExtentValues::Stripe {
                                        physical_offset: data_area_physical_offset,
                                        physical_volume_index: *physical_volume_index,
                                    },
                                };
                                logical_volume.extents.push(extent);
                            }
                        }
                        _ => {
                            return Err(keramics_core::error_trace_new!(format!(
                                "Unsupported segment: {} - unsupported type: {}",
                                segment_index, segment.segment_type
                            )));
                        }
                    }
                    logical_volume_size += segment_size;
                }
                logical_volume.size = logical_volume_size;
            }
        }
        self.file_resolver = file_resolver.clone();

        Ok(())
    }

    /// Reads the volume group from the metadata area,
    fn read_volume_group(
        &mut self,
        data_stream: &DataStreamReference,
        data_area_descriptor: &LinuxLvmDataAreaDescriptor,
    ) -> Result<LinuxLvmVolumeGroup, ErrorTrace> {
        let mut metadata_area_header: LinuxLvmMetadataAreaHeader =
            LinuxLvmMetadataAreaHeader::new();

        match metadata_area_header.read_at_position(
            data_stream,
            SeekFrom::Start(data_area_descriptor.physical_offset),
        ) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to read metadata area header at offset: {} (0x{:08x})",
                        data_area_descriptor.physical_offset, data_area_descriptor.physical_offset
                    )
                );
                return Err(error);
            }
        }
        let location_descriptor: &LinuxLvmRawLocationDescriptor =
            match metadata_area_header.location_descriptors.get(0) {
                Some(location_descriptor) => location_descriptor,
                None => {
                    return Err(keramics_core::error_trace_new!(
                        "Missing location descriptors"
                    ));
                }
            };
        let mut metadata: LinuxLvmMetadata = LinuxLvmMetadata::new();

        match metadata.read_at_position(
            data_stream,
            location_descriptor.size,
            SeekFrom::Start(data_area_descriptor.physical_offset + location_descriptor.offset),
            location_descriptor.checksum,
        ) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read metadata");
                return Err(error);
            }
        }
        match metadata.volume_group {
            Some(volume_group) => Ok(volume_group),
            None => Err(keramics_core::error_trace_new!("Missing volume group")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use crate::os_file_resolver::open_os_file_resolver;

    use crate::tests::get_test_data_path;

    fn get_volume_system() -> Result<LinuxLvmVolumeSystem, ErrorTrace> {
        let mut volume_system: LinuxLvmVolumeSystem = LinuxLvmVolumeSystem::new();

        let path_string: String = get_test_data_path("linuxlvm");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let file_resolver: FileResolverReference = open_os_file_resolver(&path_buf)?;

        let data_file_descriptors: [LinuxLvmDataFileDescriptor; 1] =
            [LinuxLvmDataFileDescriptor::new(
                PathComponent::from("lvm2.raw"),
                0,
            )];
        volume_system.open(&file_resolver, &data_file_descriptors)?;

        Ok(volume_system)
    }

    #[test]
    fn test_get_bytes_per_sector() -> Result<(), ErrorTrace> {
        let volume_system: LinuxLvmVolumeSystem = get_volume_system()?;

        let bytes_per_sector: u16 = volume_system.get_bytes_per_sector();
        assert_eq!(bytes_per_sector, 512);

        Ok(())
    }

    // TODO: add tests for get_identifier
    // TODO: add tests for get_name
    // TODO: add tests for get_number_of_physical_volumes
    // TODO: add tests for get_number_of_volumes
    // TODO: add tests for get_volume_by_index

    #[test]
    fn test_open() -> Result<(), ErrorTrace> {
        let mut volume_system: LinuxLvmVolumeSystem = LinuxLvmVolumeSystem::new();

        let path_string: String = get_test_data_path("linuxlvm");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let file_resolver: FileResolverReference = open_os_file_resolver(&path_buf)?;

        assert!(volume_system.volume_group.is_none());

        let data_file_descriptors: [LinuxLvmDataFileDescriptor; 1] =
            [LinuxLvmDataFileDescriptor::new(
                PathComponent::from("lvm2.raw"),
                0,
            )];
        volume_system.open(&file_resolver, &data_file_descriptors)?;
        assert!(volume_system.volume_group.is_some());

        Ok(())
    }
}
