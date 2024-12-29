/* Copyright 2024 Joachim Metz <joachim.metz@gmail.com>
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

use std::cmp::min;
use std::collections::HashSet;
use std::io;
use std::io::SeekFrom;

use crate::sigscan::{BuildError, PatternType, ScanContext, Scanner, Signature};
use crate::vfs::VfsDataStreamReference;

use super::enums::FormatIdentifier;

/// Format scanner.
pub struct FormatScanner {
    /// Signature scanner.
    signature_scanner: Scanner,
}

impl FormatScanner {
    /// Creates a new format scanner.
    pub fn new() -> Self {
        Self {
            signature_scanner: Scanner::new(),
        }
    }

    /// Adds Apple Partition Map (APM) signatures.
    pub fn add_apm_signatures(&mut self) {
        // APM signature.
        // Note that technically "PM" at offset 512 is the Apple Partion Map
        // signature but using the partition type is less error prone.
        self.signature_scanner.add_signature(Signature::new(
            "apm1",
            PatternType::BoundToStart,
            560,
            &[
                0x41, 0x70, 0x70, 0x6c, 0x65, 0x5f, 0x70, 0x61, 0x72, 0x74, 0x69, 0x74, 0x69, 0x6f,
                0x6e, 0x5f, 0x6d, 0x61, 0x70, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00,
            ],
        ));
    }

    /// Adds Extended File System (ext) signatures.
    pub fn add_ext_signatures(&mut self) {
        // Signature in superblock.
        self.signature_scanner.add_signature(Signature::new(
            "ext1",
            PatternType::BoundToStart,
            1080,
            &[0x53, 0xef],
        ));
    }

    /// Adds GUID Partition Table (GPT) signatures.
    pub fn add_gpt_signatures(&mut self) {
        // Signature for 512 bytes per sector.
        self.signature_scanner.add_signature(Signature::new(
            "gpt1",
            PatternType::BoundToStart,
            512,
            &[0x45, 0x46, 0x49, 0x20, 0x50, 0x41, 0x52, 0x54],
        ));
        // Signature for 1024 bytes per sector.
        self.signature_scanner.add_signature(Signature::new(
            "gpt2",
            PatternType::BoundToStart,
            1024,
            &[0x45, 0x46, 0x49, 0x20, 0x50, 0x41, 0x52, 0x54],
        ));
        // Signature for 2048 bytes per sector.
        self.signature_scanner.add_signature(Signature::new(
            "gpt3",
            PatternType::BoundToStart,
            2048,
            &[0x45, 0x46, 0x49, 0x20, 0x50, 0x41, 0x52, 0x54],
        ));
        // Signature for 4096 bytes per sector.
        self.signature_scanner.add_signature(Signature::new(
            "gpt4",
            PatternType::BoundToStart,
            4096,
            &[0x45, 0x46, 0x49, 0x20, 0x50, 0x41, 0x52, 0x54],
        ));
    }

    /// Adds QEMU Copy-On-Write (QCOW) signatures.
    pub fn add_qcow_signatures(&mut self) {
        // Version 1 signature and version in header.
        self.signature_scanner.add_signature(Signature::new(
            "qcow1",
            PatternType::BoundToStart,
            0,
            &[0x51, 0x46, 0x49, 0xfb, 0x00, 0x00, 0x00, 0x01],
        ));
        // Version 2 signature and version in header.
        self.signature_scanner.add_signature(Signature::new(
            "qcow2",
            PatternType::BoundToStart,
            0,
            &[0x51, 0x46, 0x49, 0xfb, 0x00, 0x00, 0x00, 0x02],
        ));
        // Version 3 signature and version in header.
        self.signature_scanner.add_signature(Signature::new(
            "qcow3",
            PatternType::BoundToStart,
            0,
            &[0x51, 0x46, 0x49, 0xfb, 0x00, 0x00, 0x00, 0x03],
        ));
    }

    /// Adds Mac OS sparse image (.sparseimage) signatures.
    pub fn add_sparseimage_signatures(&mut self) {
        // Signature in header.
        self.signature_scanner.add_signature(Signature::new(
            "sparseimage1",
            PatternType::BoundToStart,
            0,
            &[0x73, 0x70, 0x72, 0x73],
        ));
    }

    /// Adds Universal Disk Image Format (UDIF) (signatures.
    pub fn add_udif_signatures(&mut self) {
        // Signature in footer.
        self.signature_scanner.add_signature(Signature::new(
            "udif1",
            PatternType::BoundToEnd,
            512,
            &[0x6b, 0x6f, 0x6c, 0x79],
        ));
    }

    /// Adds Virtual Hard Disk (VHD) signatures.
    pub fn add_vhd_signatures(&mut self) {
        // Signature in footer.
        self.signature_scanner.add_signature(Signature::new(
            "vhd1",
            PatternType::BoundToEnd,
            512,
            &[0x63, 0x6f, 0x6e, 0x65, 0x63, 0x74, 0x69, 0x78],
        ));
    }

    /// Adds Virtual Hard Disk version 2 (VHDX) signatures.
    pub fn add_vhdx_signatures(&mut self) {
        // Signature in header.
        self.signature_scanner.add_signature(Signature::new(
            "vhdx1",
            PatternType::BoundToStart,
            0,
            &[0x76, 0x68, 0x64, 0x78, 0x66, 0x69, 0x6c, 0x65],
        ));
    }

    /// Builds the format signature scanner.
    pub fn build(&mut self) -> Result<(), BuildError> {
        self.signature_scanner.build()
    }

    /// Scans a data stream for format signatures.
    pub fn scan_data_stream(
        &self,
        data_stream: &VfsDataStreamReference,
    ) -> io::Result<HashSet<FormatIdentifier>> {
        let data_size: u64 = match data_stream.with_write_lock() {
            Ok(mut data_stream) => data_stream.get_size()?,
            Err(error) => return Err(crate::error_to_io_error!(error)),
        };
        let mut scan_context: ScanContext = ScanContext::new(&self.signature_scanner, data_size);

        let (range_start_offset, range_end_offset): (u64, u64) = scan_context.get_header_range();
        let range_size: usize = (range_end_offset - range_start_offset) as usize;

        let read_size: usize = min(range_size, data_size as usize);
        let mut data: Vec<u8> = vec![0; read_size];

        match data_stream.with_write_lock() {
            Ok(mut data_stream) => data_stream
                .read_exact_at_position(&mut data, SeekFrom::Start(range_start_offset))?,
            Err(error) => return Err(crate::error_to_io_error!(error)),
        };
        scan_context.scan_buffer(&data);

        let (range_start_offset, range_end_offset): (u64, u64) = scan_context.get_footer_range();
        let range_size: usize = (range_end_offset - range_start_offset) as usize;

        let read_size: usize = min(range_size, data_size as usize);
        let mut data: Vec<u8> = vec![0; read_size];

        match data_stream.with_write_lock() {
            Ok(mut data_stream) => data_stream
                .read_exact_at_position(&mut data, SeekFrom::Start(range_start_offset))?,
            Err(error) => return Err(crate::error_to_io_error!(error)),
        };
        scan_context.data_offset = range_start_offset;
        scan_context.scan_buffer(&data);

        let mut scan_results: HashSet<FormatIdentifier> = HashSet::new();
        for signature in scan_context.results.values() {
            let format_identifier: FormatIdentifier = match signature.identifier.as_str() {
                "apm1" => FormatIdentifier::Apm,
                "ext1" => FormatIdentifier::Ext,
                "gpt1" | "gpt2" | "gpt3" | "gpt4" => FormatIdentifier::Gpt,
                "qcow1" | "qcow2" | "qcow3" => FormatIdentifier::Qcow,
                "sparseimage1" => FormatIdentifier::SparseImage,
                "udif1" => FormatIdentifier::Udif,
                "vhd1" => FormatIdentifier::Vhd,
                "vhdx1" => FormatIdentifier::Vhdx,
                _ => FormatIdentifier::Unknown,
            };
            scan_results.insert(format_identifier);
        }
        Ok(scan_results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::vfs::{VfsContext, VfsPath, VfsPathType};

    #[test]
    fn test_scan_data_stream() -> io::Result<()> {
        let mut format_scanner: FormatScanner = FormatScanner::new();
        format_scanner.add_apm_signatures();
        format_scanner.add_ext_signatures();
        format_scanner.add_gpt_signatures();
        format_scanner.add_qcow_signatures();
        format_scanner.add_sparseimage_signatures();
        format_scanner.add_udif_signatures();
        format_scanner.add_vhd_signatures();
        format_scanner.add_vhdx_signatures();

        match format_scanner.build() {
            Ok(_) => {}
            Err(error) => return Err(crate::error_to_io_error!(error)),
        };
        let mut vfs_context: VfsContext = VfsContext::new();

        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Os, "./test_data/qcow/ext2.qcow2", None);
        let vfs_data_stream: VfsDataStreamReference =
            match vfs_context.open_data_stream(&vfs_path, None)? {
                Some(data_stream) => data_stream,
                None => {
                    return Err(io::Error::new(
                        io::ErrorKind::NotFound,
                        format!("No such file: {}", vfs_path.location),
                    ))
                }
            };
        let scan_results: HashSet<FormatIdentifier> =
            format_scanner.scan_data_stream(&vfs_data_stream)?;

        assert_eq!(scan_results.len(), 1);
        assert!(scan_results.iter().next() == Some(&FormatIdentifier::Qcow));

        Ok(())
    }
}
