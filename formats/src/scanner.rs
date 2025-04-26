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

use std::collections::HashSet;
use std::io;
use std::io::SeekFrom;

use core::DataStreamReference;
use sigscan::{BuildError, PatternType, ScanContext, Scanner, Signature};

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

    /// Adds Master Boot Record (MBR) signatures.
    pub fn add_mbr_signatures(&mut self) {
        // Signature for 512 bytes per sector.
        self.signature_scanner.add_signature(Signature::new(
            "mbr1",
            PatternType::BoundToStart,
            510,
            &[0x55, 0xaa],
        ));
        // Signature for 1024 bytes per sector.
        self.signature_scanner.add_signature(Signature::new(
            "mbr2",
            PatternType::BoundToStart,
            1022,
            &[0x55, 0xaa],
        ));
        // Signature for 2048 bytes per sector.
        self.signature_scanner.add_signature(Signature::new(
            "mbr3",
            PatternType::BoundToStart,
            2046,
            &[0x55, 0xaa],
        ));
        // Signature for 4096 bytes per sector.
        self.signature_scanner.add_signature(Signature::new(
            "mbr4",
            PatternType::BoundToStart,
            4094,
            &[0x55, 0xaa],
        ));
    }

    /// Adds New Technologies File System (NTFS) signatures.
    pub fn add_ntfs_signatures(&mut self) {
        // Signature in boot record.
        self.signature_scanner.add_signature(Signature::new(
            "ntfs1",
            PatternType::BoundToStart,
            3,
            &[0x4e, 0x54, 0x46, 0x53, 0x20, 0x20, 0x20, 0x20],
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
        data_stream: &DataStreamReference,
    ) -> io::Result<HashSet<FormatIdentifier>> {
        let data_size: u64 = match data_stream.write() {
            Ok(mut data_stream) => data_stream.get_size()?,
            Err(error) => return Err(core::error_to_io_error!(error)),
        };
        let mut scan_context: ScanContext = ScanContext::new(&self.signature_scanner, data_size);

        // The size of the header range can be larger than the size of the data stream.
        let mut data: Vec<u8> = vec![0; scan_context.header_range_size as usize];

        match data_stream.write() {
            Ok(mut data_stream) => data_stream.read_at_position(&mut data, SeekFrom::Start(0))?,
            Err(error) => return Err(core::error_to_io_error!(error)),
        };
        scan_context.data_offset = 0;
        scan_context.scan_buffer(&data);

        // The size of the footer range can be larger than the size of the data stream.
        let mut data: Vec<u8> = vec![0; scan_context.footer_range_size as usize];

        let data_offset: usize = if scan_context.footer_range_size < data_size {
            0
        } else {
            (scan_context.footer_range_size - data_size) as usize
        };
        let data_stream_offset: u64 = if scan_context.footer_range_size < data_size {
            data_size - scan_context.footer_range_size
        } else {
            0
        };
        match data_stream.write() {
            Ok(mut data_stream) => data_stream.read_at_position(
                &mut data[data_offset..],
                SeekFrom::Start(data_stream_offset),
            )?,
            Err(error) => return Err(core::error_to_io_error!(error)),
        };
        scan_context.data_offset = data_stream_offset;
        scan_context.scan_buffer(&data);

        let mut scan_results: HashSet<FormatIdentifier> = HashSet::new();
        for signature in scan_context.results.values() {
            let format_identifier: FormatIdentifier = match signature.identifier.as_str() {
                "apm1" => FormatIdentifier::Apm,
                "ext1" => FormatIdentifier::Ext,
                "gpt1" | "gpt2" | "gpt3" | "gpt4" => FormatIdentifier::Gpt,
                "mbr1" | "mbr2" | "mbr3" | "mbr4" => FormatIdentifier::Mbr,
                "ntfs1" => FormatIdentifier::Ntfs,
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

    use core::open_os_data_stream;

    #[test]
    fn test_build() -> Result<(), BuildError> {
        let mut format_scanner: FormatScanner = FormatScanner::new();
        format_scanner.add_apm_signatures();
        format_scanner.add_ext_signatures();
        format_scanner.add_gpt_signatures();
        format_scanner.add_ntfs_signatures();
        format_scanner.add_qcow_signatures();
        format_scanner.add_sparseimage_signatures();
        format_scanner.add_udif_signatures();
        format_scanner.add_vhd_signatures();
        format_scanner.add_vhdx_signatures();

        format_scanner.build()
    }

    #[test]
    fn test_scan_data_stream() -> io::Result<()> {
        let mut format_scanner: FormatScanner = FormatScanner::new();
        format_scanner.add_apm_signatures();
        format_scanner.add_ext_signatures();
        format_scanner.add_gpt_signatures();
        format_scanner.add_ntfs_signatures();
        format_scanner.add_qcow_signatures();
        format_scanner.add_sparseimage_signatures();
        format_scanner.add_udif_signatures();
        format_scanner.add_vhd_signatures();
        format_scanner.add_vhdx_signatures();

        match format_scanner.build() {
            Ok(_) => {}
            Err(error) => return Err(core::error_to_io_error!(error)),
        };

        let data_stream: DataStreamReference = open_os_data_stream("../test_data/qcow/ext2.qcow2")?;
        let scan_results: HashSet<FormatIdentifier> =
            format_scanner.scan_data_stream(&data_stream)?;

        assert_eq!(scan_results.len(), 1);
        assert!(scan_results.iter().next() == Some(&FormatIdentifier::Qcow));

        Ok(())
    }
}
