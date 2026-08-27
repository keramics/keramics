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
use std::sync::Arc;

use keramics_core::ErrorTrace;
use keramics_formats::vmdk::{VmdkCompressionMethod, VmdkDiskType, VmdkImage, VmdkImageLayer};
use keramics_formats::{FileResolverReference, PathComponent, open_os_file_resolver};
use keramics_types::ByteString;

use crate::formatters::ByteSize;

/// Information about a VMware Virtual Disk (VMDK) image layer.
struct VmdkImageLayerInfo<'a> {
    /// Image layer.
    image_layer: &'a VmdkImageLayer,
}

impl<'a> VmdkImageLayerInfo<'a> {
    const COMPRESSION_METHODS: &'static [(VmdkCompressionMethod, &'static str); 2] = &[
        (VmdkCompressionMethod::None, "Uncompressed"),
        (VmdkCompressionMethod::Zlib, "zlib"),
    ];

    const DISK_TYPES: &'static [(VmdkDiskType, &'static str); 16] = &[
        (VmdkDiskType::Custom, "Custom"),
        (VmdkDiskType::Device, "Device"),
        (VmdkDiskType::DevicePartitioned, "Device partitioned"),
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
    fn new(image_layer: &'a VmdkImageLayer) -> Self {
        Self { image_layer }
    }

    /// Retrieves the compression method as a string.
    pub fn get_compression_method_string(
        &self,
        compression_method: &VmdkCompressionMethod,
    ) -> &str {
        Self::COMPRESSION_METHODS
            .binary_search_by(|(key, _)| key.cmp(compression_method))
            .map_or_else(|_| "Unknown", |index| Self::COMPRESSION_METHODS[index].1)
    }

    /// Retrieves the disk type as a string.
    pub fn get_disk_type_string(&self, disk_type: &VmdkDiskType) -> &str {
        Self::DISK_TYPES
            .binary_search_by(|(key, _)| key.cmp(disk_type))
            .map_or_else(|_| "Unknown", |index| Self::DISK_TYPES[index].1)
    }
}

impl<'a> fmt::Display for VmdkImageLayerInfo<'a> {
    /// Formats image information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "VMware Virtual Disk (VMDK) information:")?;

        let disk_type_string: &str = self.get_disk_type_string(self.image_layer.get_disk_type());
        writeln!(formatter, "    Disk type\t\t\t\t\t: {}", disk_type_string)?;

        writeln!(
            formatter,
            "    Sectors per grain\t\t\t\t: {}",
            self.image_layer.get_sectors_per_grain()
        )?;
        let compression_method_string: &str =
            self.get_compression_method_string(self.image_layer.get_compression_method());
        writeln!(
            formatter,
            "    Compression method\t\t\t\t: {}",
            compression_method_string
        )?;
        writeln!(
            formatter,
            "    Content identifier\t\t\t\t: 0x{:08x}",
            self.image_layer.get_content_identifier()
        )?;
        let parent_content_identifier: Option<u32> =
            self.image_layer.get_parent_content_identifier();
        let parent_name: Option<&ByteString> = self.image_layer.get_parent_name();

        if parent_content_identifier.is_some() || parent_name.is_some() {
            writeln!(formatter, "    Parent information:")?;

            if let Some(content_identifier) = parent_content_identifier {
                writeln!(
                    formatter,
                    "        Content identifier\t\t\t: 0x{:08x}",
                    content_identifier
                )?;
            }
            if let Some(name) = parent_name {
                writeln!(formatter, "        Name\t\t\t\t\t: {}", name)?;
            }
        }
        writeln!(formatter, "    Media information:")?;

        let byte_size: ByteSize = ByteSize::new(self.image_layer.get_media_size(), 1024);
        writeln!(formatter, "        Media size\t\t\t\t: {}", byte_size)?;

        writeln!(
            formatter,
            "        Bytes per sector\t\t\t: {}",
            self.image_layer.get_bytes_per_sector()
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

        let vmdk_image_layer: Arc<VmdkImageLayer> =
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
        let image_layer_information: VmdkImageLayerInfo =
            VmdkImageLayerInfo::new(&vmdk_image_layer);

        print!("{}", image_layer_information);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use crate::assert_lines_eq;

    #[test]
    fn test_image_layer_information_fmt() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/vmdk/ext2.vmdk");
        let vmdk_image: VmdkImage = VmdkInfo::open_image(&path_buf)?;
        let vmdk_image_layer: Arc<VmdkImageLayer> = vmdk_image.get_layer_by_index(0)?;
        let test_struct: VmdkImageLayerInfo = VmdkImageLayerInfo::new(&vmdk_image_layer);

        let expected_string: &str = concat!(
            "VMware Virtual Disk (VMDK) information:\n",
            "    Disk type\t\t\t\t\t: Monolithic sparse\n",
            "    Sectors per grain\t\t\t\t: 128\n",
            "    Compression method\t\t\t\t: Uncompressed\n",
            "    Content identifier\t\t\t\t: 0x4c069322\n",
            "    Media information:\n",
            "        Media size\t\t\t\t: 4.0 MiB (4194304 bytes)\n",
            "        Bytes per sector\t\t\t: 512\n",
            "\n"
        );
        let string: String = test_struct.to_string();
        assert_lines_eq!(string.as_str(), expected_string);

        Ok(())
    }

    // TODO: add tests for open_image
    // TODO: add tests for print_image
}
