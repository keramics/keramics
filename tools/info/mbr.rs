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

use std::collections::HashMap;
use std::process::ExitCode;
use std::sync::Arc;

use crate::formatters;

use keramics::formats::mbr::{MbrPartition, MbrVolumeSystem};
use keramics::vfs::{VfsFileSystem, VfsPath};

/// Prints information about a MBR volume system.
pub fn print_mbr_volume_system(
    vfs_file_system: &Arc<VfsFileSystem>,
    vfs_path: &VfsPath,
) -> ExitCode {
    let mut mbr_volume_system = MbrVolumeSystem::new();

    match mbr_volume_system.open(vfs_file_system, vfs_path) {
        Ok(_) => {}
        Err(error) => {
            println!("Unable to open MBR volume system with error: {}", error);
            return ExitCode::FAILURE;
        }
    };
    let partition_types = HashMap::<u8, &'static str>::from([
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
    ]);
    println!("Master Boot Record (MBR) information:");

    // TODO: print disk identity

    println!(
        "    Bytes per sector\t\t\t: {} bytes",
        mbr_volume_system.bytes_per_sector
    );
    let number_of_partitions: usize = mbr_volume_system.get_number_of_partitions();
    println!("    Number of partitions\t\t: {}", number_of_partitions);

    for partition_index in 0..number_of_partitions {
        println!("");

        let mbr_partition: MbrPartition =
            match mbr_volume_system.get_partition_by_index(partition_index) {
                Ok(partition) => partition,
                Err(error) => {
                    println!(
                        "Unable to retrieve MBR partition: {} with error: {}",
                        partition_index, error
                    );
                    return ExitCode::FAILURE;
                }
            };
        let size_string: String = formatters::format_as_bytesize(mbr_partition.size, 1024);

        println!("Partition: {}", partition_index + 1);

        match partition_types.get(&mbr_partition.partition_type) {
            Some(partition_type_string) => {
                println!(
                    "    Type\t\t\t\t: 0x{:02x} ({})",
                    mbr_partition.partition_type, partition_type_string
                );
            }
            None => {
                println!("    Type\t\t\t\t: 0x{:02x}", mbr_partition.partition_type);
            }
        };
        println!(
            "    Offset\t\t\t\t: {} (0x{:08x})",
            mbr_partition.offset, mbr_partition.offset
        );
        println!(
            "    Size\t\t\t\t: {} ({} bytes)",
            size_string, mbr_partition.size
        );
        println!("    Flags\t\t\t\t: 0x{:02x}", mbr_partition.flags);
    }
    println!("");

    ExitCode::SUCCESS
}
