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

use std::fmt;

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_formats::mbr::{MbrPartition, MbrVolumeSystem};

use crate::formatters::ByteSize;

/// Master Boot Record (MBR) parition information.
struct MbrPartitionInfo {
    /// The partition index.
    pub index: usize,

    /// The partition type.
    pub partition_type: u8,

    /// The offset of the partition relative to start of the volume system.
    pub offset: u64,

    /// The size of the partition.
    pub size: u64,

    /// The flags.
    pub flags: u8,
}

impl MbrPartitionInfo {
    const PARTITION_TYPES: &[(u8, &'static str); 91] = &[
        (0x00, "Empty"),
        (0x01, "FAT12 (CHS)"),
        (0x02, "XENIX root"),
        (0x03, "XENIX user"),
        (0x04, "FAT16 < 32 MiB (CHS)"),
        (0x05, "Extended (CHS)"),
        (0x06, "FAT16 (CHS)"),
        (0x07, "HPFS/NTFS"),
        (0x08, "AIX"),
        (0x09, "AIX bootable"),
        (0x0a, "OS/2 Boot Manager"),
        (0x0b, "FAT32 (CHS)"),
        (0x0c, "FAT32 (LBA)"),
        (0x0e, "FAT16 (LBA)"),
        (0x0f, "Extended (LBA)"),
        (0x10, "OPUS"),
        (0x11, "Hidden FAT12 (CHS)"),
        (0x12, "Compaq diagnostics"),
        (0x14, "Hidden FAT16 < 32 MiB (CHS)"),
        (0x16, "Hidden FAT16 (CHS)"),
        (0x17, "Hidden HPFS/NTFS"),
        (0x18, "AST SmartSleep"),
        (0x1b, "Hidden FAT32 (CHS)"),
        (0x1c, "Hidden FAT32 (LBA)"),
        (0x1e, "Hidden FAT16 (LBA)"),
        (0x24, "NEC DOS"),
        (0x39, "Plan 9"),
        (0x3c, "PartitionMagic recovery"),
        (0x40, "Venix 80286"),
        (0x41, "PPC PReP Boot"),
        (0x42, "SFS / MS LDM"),
        (0x4d, "QNX4.x"),
        (0x4e, "QNX4.x 2nd part"),
        (0x4f, "QNX4.x 3rd part"),
        (0x50, "OnTrack DM"),
        (0x51, "OnTrack DM6 Aux1"),
        (0x52, "CP/M"),
        (0x53, "OnTrack DM6 Aux3"),
        (0x54, "OnTrackDM6"),
        (0x55, "EZ-Drive"),
        (0x56, "Golden Bow"),
        (0x5c, "Priam Edisk"),
        (0x61, "SpeedStor"),
        (0x63, "GNU HURD or SysV"),
        (0x64, "Novell Netware 286"),
        (0x65, "Novell Netware 386"),
        (0x70, "DiskSecure Multi-Boot"),
        (0x75, "PC/IX"),
        (0x78, "XOSL"),
        (0x80, "Old Minix"),
        (0x81, "Minix / old Linux"),
        (0x82, "Linux swap / Solaris"),
        (0x83, "Linux"),
        (0x84, "OS/2 hidden C: drive"),
        (0x85, "Linux extended"),
        (0x86, "NTFS partition set"),
        (0x87, "NTFS partition set"),
        (0x8e, "Linux LVM"),
        (0x93, "Amoeba"),
        (0x94, "Amoeba BBT"),
        (0x9f, "BSD/OS"),
        (0xa0, "IBM Thinkpad hibernation"),
        (0xa5, "FreeBSD"),
        (0xa6, "OpenBSD"),
        (0xa7, "NeXTSTEP"),
        (0xa9, "NetBSD"),
        (0xaf, "MacOS-X"),
        (0xb7, "BSDI fs"),
        (0xb8, "BSDI swap"),
        (0xbb, "Boot Wizard hidden"),
        (0xc1, "DRDOS/sec (FAT-12)"),
        (0xc4, "DRDOS/sec (FAT-16 < 32 MiB)"),
        (0xc6, "DRDOS/sec (FAT-16)"),
        (0xc7, "Syrinx"),
        (0xda, "Non-FS data"),
        (0xdb, "CP/M / CTOS"),
        (0xde, "Dell Utility"),
        (0xdf, "BootIt"),
        (0xe1, "DOS access"),
        (0xe3, "DOS R/O"),
        (0xe4, "SpeedStor"),
        (0xeb, "BeOS fs"),
        (0xee, "EFI GPT protective"),
        (0xef, "EFI System (FAT)"),
        (0xf0, "Linux/PA-RISC boot"),
        (0xf1, "SpeedStor"),
        (0xf4, "SpeedStor"),
        (0xf2, "DOS secondary"),
        (0xfd, "Linux raid autodetect"),
        (0xfe, "LANstep"),
        (0xff, "BBT"),
    ];

    /// Creates new partition information.
    fn new() -> Self {
        Self {
            index: 0,
            partition_type: 0,
            offset: 0,
            size: 0,
            flags: 0,
        }
    }

    /// Retrieves the partition type as a string.
    pub fn get_partition_type_string(&self) -> Option<&str> {
        Self::PARTITION_TYPES
            .binary_search_by(|(key, _)| key.cmp(&self.partition_type))
            .map_or_else(|_| None, |index| Some(Self::PARTITION_TYPES[index].1))
    }
}

impl fmt::Display for MbrPartitionInfo {
    /// Formats partition information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "Partition: {}", self.index + 1)?;

        match self.get_partition_type_string() {
            Some(partition_type_string) => {
                writeln!(
                    formatter,
                    "    Type\t\t\t\t\t: 0x{:02x} ({})",
                    self.partition_type, partition_type_string
                )?;
            }
            None => {
                writeln!(
                    formatter,
                    "    Type\t\t\t\t\t: 0x{:02x}",
                    self.partition_type
                )?;
            }
        };
        writeln!(
            formatter,
            "    Offset\t\t\t\t\t: {} (0x{:08x})",
            self.offset, self.offset
        )?;
        let byte_size: ByteSize = ByteSize::new(self.size, 1024);
        writeln!(formatter, "    Size\t\t\t\t\t: {}", byte_size)?;

        writeln!(formatter, "    Flags\t\t\t\t\t: 0x{:02x}", self.flags)?;

        writeln!(formatter)
    }
}

/// Information about a Master Boot Record (MBR).
pub struct MbrInfo {}

impl MbrInfo {
    /// Retrieves the partition information.
    fn get_partition_information(
        partition_index: usize,
        mbr_partition: &MbrPartition,
    ) -> MbrPartitionInfo {
        let mut partition_information: MbrPartitionInfo = MbrPartitionInfo::new();

        partition_information.index = partition_index;
        partition_information.partition_type = mbr_partition.partition_type;
        partition_information.offset = mbr_partition.offset;
        partition_information.size = mbr_partition.size;
        partition_information.flags = mbr_partition.flags;

        partition_information
    }

    /// Opens a volume system.
    pub fn open_volume_system(
        data_stream: &DataStreamReference,
    ) -> Result<MbrVolumeSystem, ErrorTrace> {
        let mut mbr_volume_system: MbrVolumeSystem = MbrVolumeSystem::new();

        match mbr_volume_system.read_data_stream(data_stream) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open MBR volume system");
                return Err(error);
            }
        }
        Ok(mbr_volume_system)
    }

    /// Prints information about a volume system.
    pub fn print_volume_system(data_stream: &DataStreamReference) -> Result<(), ErrorTrace> {
        let mbr_volume_system: MbrVolumeSystem = match Self::open_volume_system(data_stream) {
            Ok(mbr_volume_system) => mbr_volume_system,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open volume system");
                return Err(error);
            }
        };
        println!("Master Boot Record (MBR) information:");

        println!(
            "    Disk identity\t\t\t\t: 0x{:x}",
            mbr_volume_system.disk_identity
        );
        println!(
            "    Bytes per sector\t\t\t\t: {} bytes",
            mbr_volume_system.bytes_per_sector
        );
        let number_of_partitions: usize = mbr_volume_system.get_number_of_partitions();
        println!("    Number of partitions\t\t\t: {}", number_of_partitions);

        println!("");

        for (partition_index, result) in mbr_volume_system.partitions().enumerate() {
            let mbr_partition: MbrPartition = match result {
                Ok(mbr_partition) => mbr_partition,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to retrieve partition: {}", partition_index)
                    );
                    return Err(error);
                }
            };
            let partition_info: MbrPartitionInfo =
                Self::get_partition_information(partition_index, &mbr_partition);

            print!("{}", partition_info);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;

    #[test]
    fn test_partition_information_fmt() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/mbr/mbr.raw");
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let mbr_volume_system: MbrVolumeSystem = MbrInfo::open_volume_system(&data_stream)?;

        let mbr_partition: MbrPartition = mbr_volume_system.get_partition_by_index(0)?;
        let test_struct: MbrPartitionInfo = MbrInfo::get_partition_information(0, &mbr_partition);

        let string: String = test_struct.to_string();
        let expected_string: &str = concat!(
            "Partition: 1\n",
            "    Type\t\t\t\t\t: 0x83 (Linux)\n",
            "    Offset\t\t\t\t\t: 512 (0x00000200)\n",
            "    Size\t\t\t\t\t: 1.0 MiB (1049088 bytes)\n",
            "    Flags\t\t\t\t\t: 0x00\n",
            "\n"
        );
        assert_eq!(string, expected_string);

        Ok(())
    }

    #[test]
    fn test_get_partition_information() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/mbr/mbr.raw");
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let mbr_volume_system: MbrVolumeSystem = MbrInfo::open_volume_system(&data_stream)?;

        let mbr_partition: MbrPartition = mbr_volume_system.get_partition_by_index(0)?;
        let test_struct: MbrPartitionInfo = MbrInfo::get_partition_information(0, &mbr_partition);

        assert_eq!(test_struct.index, 0);
        assert_eq!(test_struct.partition_type, 0x83);
        assert_eq!(test_struct.offset, 512);
        assert_eq!(test_struct.size, 1049088);

        Ok(())
    }

    // TODO: add tests for open_volume_system
    // TODO: add tests for print_volume_system
}
