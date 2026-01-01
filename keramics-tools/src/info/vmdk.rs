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

use std::fmt;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use keramics_core::ErrorTrace;
use keramics_formats::vmdk::{VmdkCompressionMethod, VmdkDiskType, VmdkImage, VmdkImageLayer};
use keramics_formats::{FileResolverReference, PathComponent, open_os_file_resolver};
use keramics_types::ByteString;

use crate::formatters::ByteSize;

/// Information about a VMware Virtual Disk (VMDK) image layer.
struct VmdkImageLayerInfo {
    /// Disk type.
    pub disk_type: VmdkDiskType,

    /// Sectors per grain.
    pub sectors_per_grain: u64,

    /// Compression method.
    pub compression_method: VmdkCompressionMethod,

    /// Content identifier.
    pub content_identifier: u32,

    /// Parent content identifier.
    pub parent_content_identifier: Option<u32>,

    /// Parent name.
    pub parent_name: Option<ByteString>,

    /// Media size.
    pub media_size: u64,

    /// Bytes per sector.
    pub bytes_per_sector: u16,
}

impl VmdkImageLayerInfo {
    const COMPRESSION_METHODS: &[(VmdkCompressionMethod, &'static str); 2] = &[
        (VmdkCompressionMethod::None, "Uncompressed"),
        (VmdkCompressionMethod::Zlib, "zlib"),
    ];

    const DISK_TYPES: &[(VmdkDiskType, &'static str); 16] = &[
        (VmdkDiskType::Custom, "Custom"),
        (VmdkDiskType::Device, "Device"),
        (VmdkDiskType::DevicePartitioned, "Device paritioned"),
        (VmdkDiskType::Flat2GbExtent, "2GB extent flat"),
        (VmdkDiskType::MonolithicFlat, "Monolithic flat"),
        (VmdkDiskType::MonolithicSparse, "Monolithic sparse"),
        (VmdkDiskType::Sparse2GbExtent, "2GB extent sparse"),
        (VmdkDiskType::StreamOptimized, "Stream optimized"),
        (VmdkDiskType::VmfsFlat, "VMFS flat"),
        (
            VmdkDiskType::VmfsFlatPreAllocated,
            "VMFS flat (pre-allocated)",
        ),
        (VmdkDiskType::VmfsFlatZeroed, "VMFS flat (zeroed)"),
        (VmdkDiskType::VmfsRaw, "VMFS raw"),
        (VmdkDiskType::VmfsRdm, "VMFS RDM"),
        (VmdkDiskType::VmfsRdmp, "VMFS RDMP"),
        (VmdkDiskType::VmfsSparse, "VMFS sparse"),
        (VmdkDiskType::VmfsSparseThin, "VMFS sparse (thin)"),
    ];

    /// Creates new image information.
    fn new() -> Self {
        Self {
            disk_type: VmdkDiskType::Unknown,
            sectors_per_grain: 0,
            compression_method: VmdkCompressionMethod::None,
            content_identifier: 0,
            parent_content_identifier: None,
            parent_name: None,
            media_size: 0,
            bytes_per_sector: 0,
        }
    }

    /// Retrieves the compression method as a string.
    pub fn get_compression_method_string(&self) -> &str {
        Self::COMPRESSION_METHODS
            .binary_search_by(|(key, _)| key.cmp(&self.compression_method))
            .map_or_else(|_| "Unknown", |index| Self::COMPRESSION_METHODS[index].1)
    }

    /// Retrieves the disk type as a string.
    pub fn get_disk_type_string(&self) -> &str {
        Self::DISK_TYPES
            .binary_search_by(|(key, _)| key.cmp(&self.disk_type))
            .map_or_else(|_| "Unknown", |index| Self::DISK_TYPES[index].1)
    }
}

impl fmt::Display for VmdkImageLayerInfo {
    /// Formats image information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "VMware Virtual Disk (VMDK) information:")?;

        let disk_type_string: &str = self.get_disk_type_string();
        writeln!(formatter, "    Disk type\t\t\t\t\t: {}", disk_type_string)?;

        writeln!(
            formatter,
            "    Sectors per grain\t\t\t\t: {}",
            self.sectors_per_grain
        )?;
        let compression_method_string: &str = self.get_compression_method_string();
        writeln!(
            formatter,
            "    Compression method\t\t\t\t: {}",
            compression_method_string
        )?;
        writeln!(
            formatter,
            "    Content identifier\t\t\t\t: 0x{:08x}",
            self.content_identifier
        )?;
        if self.parent_content_identifier.is_some() || self.parent_name.is_some() {
            writeln!(formatter, "    Parent information:")?;

            if let Some(parent_content_identifier) = self.parent_content_identifier {
                writeln!(
                    formatter,
                    "        Content identifier\t\t\t: 0x{:08x}",
                    parent_content_identifier
                )?;
            }
            if let Some(parent_name) = &self.parent_name {
                writeln!(formatter, "        Name\t\t\t\t\t: {}", parent_name)?;
            }
        }
        writeln!(formatter, "    Media information:")?;

        let byte_size: ByteSize = ByteSize::new(self.media_size, 1024);
        writeln!(formatter, "        Media size\t\t\t\t: {}", byte_size)?;

        writeln!(
            formatter,
            "        Bytes per sector\t\t\t: {} bytes",
            self.bytes_per_sector
        )?;

        // TODO: print number of extents

        // TODO: print extents
        // TODO: print extent file name
        // TODO: print extent file type
        // TODO: print extent start offset
        // TODO: print extent size

        writeln!(formatter)
    }
}

/// Information about a VMware Virtual Disk (VMDK) image.
pub struct VmdkInfo {}

impl VmdkInfo {
    /// Retrieves the image layer information.
    fn get_image_layer_information(vmdk_image_layer: &VmdkImageLayer) -> VmdkImageLayerInfo {
        let mut image_layer_information: VmdkImageLayerInfo = VmdkImageLayerInfo::new();

        image_layer_information.disk_type = vmdk_image_layer.disk_type.clone();
        image_layer_information.sectors_per_grain = vmdk_image_layer.sectors_per_grain;
        image_layer_information.compression_method = vmdk_image_layer.compression_method.clone();
        image_layer_information.content_identifier = vmdk_image_layer.content_identifier;
        image_layer_information.parent_content_identifier =
            vmdk_image_layer.parent_content_identifier;
        image_layer_information.parent_name = vmdk_image_layer.parent_name.clone();
        image_layer_information.media_size = vmdk_image_layer.media_size;
        image_layer_information.bytes_per_sector = vmdk_image_layer.bytes_per_sector;

        image_layer_information
    }

    /// Opens an image.
    fn open_image(path_buf: &PathBuf) -> Result<VmdkImage, ErrorTrace> {
        let mut base_path: PathBuf = path_buf.clone();
        base_path.pop();

        let file_resolver: FileResolverReference = match open_os_file_resolver(&base_path) {
            Ok(file_resolver) => file_resolver,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to create file resolver");
                return Err(error);
            }
        };
        let mut vmdk_image: VmdkImage = VmdkImage::new();

        let file_name: PathComponent = match path_buf.file_name() {
            Some(file_name) => match file_name.to_str() {
                Some(file_name) => PathComponent::from(file_name),
                None => {
                    return Err(keramics_core::error_trace_new!("Unsupported file name"));
                }
            },
            None => {
                return Err(keramics_core::error_trace_new!("Missing file name"));
            }
        };
        match vmdk_image.open(&file_resolver, &file_name) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open VMDK image");
                return Err(error);
            }
        }
        Ok(vmdk_image)
    }

    /// Prints information about an image.
    pub fn print_image(path_buf: &PathBuf) -> Result<(), ErrorTrace> {
        let vmdk_image: VmdkImage = match Self::open_image(path_buf) {
            Ok(vmdk_image) => vmdk_image,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open image");
                return Err(error);
            }
        };
        let number_of_layers: usize = vmdk_image.get_number_of_layers();

        let vmdk_image_layer: Arc<RwLock<VmdkImageLayer>> =
            match vmdk_image.get_layer_by_index(number_of_layers - 1) {
                Ok(image_layer) => image_layer,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to retrieve image layer: {}", number_of_layers - 1)
                    );
                    return Err(error);
                }
            };
        let image_layer_information: VmdkImageLayerInfo = match vmdk_image_layer.read() {
            Ok(image_layer) => Self::get_image_layer_information(&image_layer),
            Err(_) => {
                return Err(keramics_core::error_trace_new!(
                    "Unable to obtain read lock on image layer"
                ));
            }
        };
        print!("{}", image_layer_information);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    #[test]
    fn test_image_layer_information_fmt() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/vmdk/ext2.vmdk");
        let vmdk_image: VmdkImage = VmdkInfo::open_image(&path_buf)?;
        let vmdk_image_layer: Arc<RwLock<VmdkImageLayer>> = vmdk_image.get_layer_by_index(0)?;
        let test_struct: VmdkImageLayerInfo = match vmdk_image_layer.read() {
            Ok(image_layer) => VmdkInfo::get_image_layer_information(&image_layer),
            Err(_) => {
                return Err(keramics_core::error_trace_new!(
                    "Unable to obtain read lock on image layer"
                ));
            }
        };

        let string: String = test_struct.to_string();
        let expected_string: &str = concat!(
            "VMware Virtual Disk (VMDK) information:\n",
            "    Disk type\t\t\t\t\t: Monolithic sparse\n",
            "    Sectors per grain\t\t\t\t: 128\n",
            "    Compression method\t\t\t\t: Uncompressed\n",
            "    Content identifier\t\t\t\t: 0x4c069322\n",
            "    Media information:\n",
            "        Media size\t\t\t\t: 4.0 MiB (4194304 bytes)\n",
            "        Bytes per sector\t\t\t: 512 bytes\n",
            "\n"
        );
        assert_eq!(string, expected_string);

        Ok(())
    }

    #[test]
    fn test_get_image_layer_information() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/vmdk/ext2.vmdk");
        let vmdk_image: VmdkImage = VmdkInfo::open_image(&path_buf)?;
        let vmdk_image_layer: Arc<RwLock<VmdkImageLayer>> = vmdk_image.get_layer_by_index(0)?;
        let test_struct: VmdkImageLayerInfo = match vmdk_image_layer.read() {
            Ok(image_layer) => VmdkInfo::get_image_layer_information(&image_layer),
            Err(_) => {
                return Err(keramics_core::error_trace_new!(
                    "Unable to obtain read lock on image layer"
                ));
            }
        };

        assert_eq!(test_struct.disk_type, VmdkDiskType::MonolithicSparse);
        assert_eq!(test_struct.sectors_per_grain, 128);
        assert_eq!(test_struct.compression_method, VmdkCompressionMethod::None);
        assert_eq!(test_struct.content_identifier, 0x4c069322);
        assert_eq!(test_struct.parent_content_identifier, None);
        assert_eq!(test_struct.parent_name, None);
        assert_eq!(test_struct.media_size, 4194304);
        assert_eq!(test_struct.bytes_per_sector, 512);

        Ok(())
    }

    // TODO: add tests for open_image
    // TODO: add tests for print_image
}
