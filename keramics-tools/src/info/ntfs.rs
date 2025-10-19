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

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_datetime::DateTime;
use keramics_formats::ntfs::constants::*;
use keramics_formats::ntfs::{
    NtfsAttribute, NtfsAttributeListEntry, NtfsDataFork, NtfsFileEntry, NtfsFileSystem, NtfsPath,
};

use crate::formatters::format_as_bytesize;

/// Retrieves the string representation of a date and time value.
fn get_date_time_string(date_time: &DateTime) -> Result<String, ErrorTrace> {
    match date_time {
        DateTime::NotSet => Ok(String::from("Not set (0)")),
        DateTime::Filetime(filetime) => Ok(filetime.to_iso8601_string()),
        _ => return Err(keramics_core::error_trace_new!("Unsupported date time")),
    }
}

/// Retrieves string representations of file attribute flags.
fn get_file_attribute_flags_strings(flags: u32) -> Vec<String> {
    let mut flags_strings: Vec<String> = Vec::new();
    if flags & 0x00000001 != 0 {
        let flag_string: String =
            String::from("0x00000001: Is read-only (FILE_ATTRIBUTE_READ_ONLY)");
        flags_strings.push(flag_string);
    }
    if flags & 0x00000002 != 0 {
        let flag_string: String = String::from("0x00000002: Is hidden (FILE_ATTRIBUTE_HIDDEN)");
        flags_strings.push(flag_string);
    }
    if flags & 0x00000004 != 0 {
        let flag_string: String = String::from("0x00000004: Is system (FILE_ATTRIBUTE_SYSTEM)");
        flags_strings.push(flag_string);
    }

    if flags & 0x00000010 != 0 {
        let flag_string: String =
            String::from("0x00000010: Is directory (FILE_ATTRIBUTE_DIRECTORY)");
        flags_strings.push(flag_string);
    }
    if flags & 0x00000020 != 0 {
        let flag_string: String =
            String::from("0x00000020: Should be archived (FILE_ATTRIBUTE_ARCHIVE)");
        flags_strings.push(flag_string);
    }
    if flags & 0x00000040 != 0 {
        let flag_string: String = String::from("0x00000040: Is device (FILE_ATTRIBUTE_DEVICE)");
        flags_strings.push(flag_string);
    }
    if flags & 0x00000080 != 0 {
        let flag_string: String = String::from("0x00000080: Is normal (FILE_ATTRIBUTE_NORMAL)");
        flags_strings.push(flag_string);
    }
    if flags & 0x00000100 != 0 {
        let flag_string: String =
            String::from("0x00000100: Is temporary (FILE_ATTRIBUTE_TEMPORARY)");
        flags_strings.push(flag_string);
    }
    if flags & 0x00000200 != 0 {
        let flag_string: String =
            String::from("0x00000200: Is a sparse file (FILE_ATTRIBUTE_SPARSE_FILE)");
        flags_strings.push(flag_string);
    }
    if flags & 0x00000400 != 0 {
        let flag_string: String = String::from(
            "0x00000400: Is a reparse point or symbolic link (FILE_ATTRIBUTE_FLAG_REPARSE_POINT)",
        );
        flags_strings.push(flag_string);
    }
    if flags & 0x00000800 != 0 {
        let flag_string: String =
            String::from("0x00000800: Is compressed (FILE_ATTRIBUTE_COMPRESSED)");
        flags_strings.push(flag_string);
    }
    if flags & 0x00001000 != 0 {
        let flag_string: String = String::from("0x00001000: Is offline (FILE_ATTRIBUTE_OFFLINE)");
        flags_strings.push(flag_string);
    }
    if flags & 0x00002000 != 0 {
        let flag_string: String = String::from(
            "0x00002000: Content should not be indexed (FILE_ATTRIBUTE_NOT_CONTENT_INDEXED)",
        );
        flags_strings.push(flag_string);
    }
    if flags & 0x00004000 != 0 {
        let flag_string: String =
            String::from("0x00004000: Is encrypted (FILE_ATTRIBUTE_ENCRYPTED)");
        flags_strings.push(flag_string);
    }

    if flags & 0x00010000 != 0 {
        let flag_string: String = String::from("0x00010000: Is virtual (FILE_ATTRIBUTE_VIRTUAL)");
        flags_strings.push(flag_string);
    }

    if flags & 0x20000000 != 0 {
        let flag_string: String = String::from("0x20000000: UNKNOWN (Is index view)");
        flags_strings.push(flag_string);
    }
    flags_strings
}

/// Retrieves string representations of NTFS volume flags.
fn get_ntfs_volume_flags_strings(flags: u16) -> Vec<String> {
    let mut flags_strings: Vec<String> = Vec::new();
    if flags & 0x0001 != 0 {
        let flag_string: String = String::from("0x0001: Is dirty (VOLUME_IS_DIRTY)");
        flags_strings.push(flag_string);
    }
    if flags & 0x0002 != 0 {
        let flag_string: String =
            String::from("0x0002: Re-size journal ($LogFile) (VOLUME_RESIZE_LOG_FILE)");
        flags_strings.push(flag_string);
    }
    if flags & 0x0004 != 0 {
        let flag_string: String =
            String::from("0x0004: Mounted on Windows NT 4 (VOLUME_MOUNTED_ON_NT4)");
        flags_strings.push(flag_string);
    }
    if flags & 0x0008 != 0 {
        let flag_string: String = String::from("0x0008: Is dirty (VOLUME_IS_DIRTY)");
        flags_strings.push(flag_string);
    }
    if flags & 0x0010 != 0 {
        let flag_string: String =
            String::from("0x0010: Delete USN in progress (VOLUME_DELETE_USN_UNDERWAY)");
        flags_strings.push(flag_string);
    }
    if flags & 0x0020 != 0 {
        let flag_string: String =
            String::from("0x0020: Repair object identifiers (VOLUME_REPAIR_OBJECT_ID)");
        flags_strings.push(flag_string);
    }

    if flags & 0x4000 != 0 {
        let flag_string: String =
            String::from("0x4000: chkdsk in progress (VOLUME_CHKDSK_UNDERWAY)");
        flags_strings.push(flag_string);
    }
    if flags & 0x8000 != 0 {
        let flag_string: String =
            String::from("0x8000: Modified by chkdsk (VOLUME_MODIFIED_BY_CHKDSK)");
        flags_strings.push(flag_string);
    }
    flags_strings
}

/// Information about a New Technologies File System (NTFS).
pub struct NtfsInfo {}

impl NtfsInfo {
    /// Prints information about an attribute.
    fn print_attribute(attribute: &NtfsAttribute) -> Result<(), ErrorTrace> {
        let attribute_types = HashMap::<u32, &'static str>::from([
            (
                NTFS_ATTRIBUTE_TYPE_STANDARD_INFORMATION,
                "$STANDARD_INFORMATION",
            ),
            (NTFS_ATTRIBUTE_TYPE_ATTRIBUTE_LIST, "$ATTRIBUTE_LIST"),
            (NTFS_ATTRIBUTE_TYPE_FILE_NAME, "$FILE_NAME"),
            (NTFS_ATTRIBUTE_TYPE_OBJECT_IDENTIFIER, "$OBJECT_ID"),
            (
                NTFS_ATTRIBUTE_TYPE_SECURITY_DESCRIPTOR,
                "$SECURITY_DESCRIPTOR",
            ),
            (NTFS_ATTRIBUTE_TYPE_VOLUME_NAME, "$VOLUME_NAME"),
            (
                NTFS_ATTRIBUTE_TYPE_VOLUME_INFORMATION,
                "$VOLUME_INFORMATION",
            ),
            (NTFS_ATTRIBUTE_TYPE_DATA, "$DATA"),
            (NTFS_ATTRIBUTE_TYPE_INDEX_ROOT, "$INDEX_ROOT"),
            (NTFS_ATTRIBUTE_TYPE_INDEX_ALLOCATION, "$INDEX_ALLOCATION"),
            (NTFS_ATTRIBUTE_TYPE_BITMAP, "$BITMAP"),
            (NTFS_ATTRIBUTE_TYPE_REPARSE_POINT, "$REPARSE_POINT"),
            (NTFS_ATTRIBUTE_TYPE_EXTENDED_INFORMATION, "$EA_INFORMATION"),
            (NTFS_ATTRIBUTE_TYPE_EXTENDED, "$EA"),
            (NTFS_ATTRIBUTE_TYPE_PROPERTY_SET, "$PROPERTY_SET"),
            (
                NTFS_ATTRIBUTE_TYPE_LOGGED_UTILITY_STREAM,
                "$LOGGED_UTILITY_STREAM",
            ),
        ]);
        let attribute_type: u32 = attribute.get_attribute_type();
        match attribute_types.get(&attribute_type) {
            Some(attribute_type_string) => println!(
                "Attribute: {} (0x{:08x})",
                attribute_type_string, attribute_type
            ),
            None => println!("Attribute: 0x{:08x}", attribute_type),
        };
        match attribute {
            NtfsAttribute::AttributeList { attribute_list } => {
                let number_of_entries: usize = attribute_list.entries.len();
                println!("    Number of entries\t\t\t: {}", number_of_entries);

                for entry_index in 0..number_of_entries {
                    let entry: &NtfsAttributeListEntry = &attribute_list.entries[entry_index];
                    match attribute_types.get(&entry.attribute_type) {
                        Some(attribute_type_string) => {
                            println!(
                                "    Entry: {}\t\t\t\t: {} (0x{:08x}) with file reference: {}-{}",
                                entry_index + 1,
                                attribute_type_string,
                                entry.attribute_type,
                                entry.file_reference >> 48,
                                entry.file_reference & 0x0000ffffffffffff,
                            );
                        }
                        None => {
                            println!(
                                "    Entry: {}\t\t\t\t: 0x{:08x} with file reference: {}-{}",
                                entry_index + 1,
                                entry.attribute_type,
                                entry.file_reference >> 48,
                                entry.file_reference & 0x0000ffffffffffff,
                            );
                        }
                    };
                }
                println!("");
            }
            // TODO: add support for $EA
            // TODO: add support for $EA_INFORMATION
            NtfsAttribute::FileName { file_name } => {
                let name_spaces = HashMap::<u8, &'static str>::from([
                    (0, "POSIX"),
                    (1, "Windows"),
                    (2, "DOS"),
                    (3, "DOS and Windows"),
                ]);
                match name_spaces.get(&file_name.name_space) {
                    Some(name_space_string) => println!(
                        "    Name space\t\t\t\t: {} ({})",
                        name_space_string, file_name.name_space
                    ),
                    None => println!("    Name space\t\t\t\t: {}", file_name.name_space),
                };
                println!("    Name\t\t\t\t: {}", file_name.name.to_string());

                if file_name.parent_file_reference == 0 {
                    println!("    Parent file reference\t\t: Not set (0)");
                } else {
                    println!(
                        "    Parent file reference\t\t: {}-{}",
                        file_name.parent_file_reference & 0x0000ffffffffffff,
                        file_name.parent_file_reference >> 48
                    );
                }
                let date_time_string: String = get_date_time_string(&file_name.creation_time)?;
                println!("    Creation time\t\t\t: {}", date_time_string);

                let date_time_string: String = get_date_time_string(&file_name.modification_time)?;
                println!("    Modification time\t\t\t: {}", date_time_string);

                let date_time_string: String = get_date_time_string(&file_name.access_time)?;
                println!("    Access time\t\t\t\t: {}", date_time_string);

                let date_time_string: String =
                    get_date_time_string(&file_name.entry_modification_time)?;
                println!("    Entry modification time\t\t: {}", date_time_string);

                println!(
                    "    File attribute flags\t\t: 0x{:08x}",
                    file_name.file_attribute_flags
                );
                let flags_strings: Vec<String> =
                    get_file_attribute_flags_strings(file_name.file_attribute_flags);
                println!(
                    "{}",
                    flags_strings
                        .iter()
                        .map(|string| format!("        {}", string))
                        .collect::<Vec<String>>()
                        .join("\n")
                );
                if file_name.file_attribute_flags != 0 {
                    println!("");
                }
            }
            // TODO: add support for $BITMAP, $DATA, $INDEX_ALLOCATION, $INDEX_ROOT
            NtfsAttribute::Generic { mft_attribute } => {
                match &mft_attribute.name {
                    Some(name) => println!("    Attribute name\t\t\t: {}", name.to_string()),
                    None => {}
                };
                if mft_attribute.data_size < 1024 {
                    println!("    Data size\t\t\t\t: {} bytes", mft_attribute.data_size);
                } else {
                    let data_size_string: String =
                        format_as_bytesize(mft_attribute.data_size, 1024);
                    println!(
                        "    Data size\t\t\t\t: {} ({} bytes)",
                        data_size_string, mft_attribute.data_size
                    );
                }
                if attribute_type == NTFS_ATTRIBUTE_TYPE_DATA {
                    println!("    Data flags\t\t\t\t: 0x{:04x}", mft_attribute.data_flags);
                }
                if mft_attribute.data_cluster_groups.len() > 0 {
                    let string_parts: Vec<String> = mft_attribute
                        .data_cluster_groups
                        .iter()
                        .map(|cluster_group| {
                            format!("{}-{}", cluster_group.first_vcn, cluster_group.last_vcn)
                        })
                        .collect::<Vec<String>>();
                    println!("    VCNs\t\t\t\t: [{}]", string_parts.join(", "));
                }
                println!("");
            }
            // TODO: add support for $LOGGED_UTILITY_STREAM
            // TODO: add support for $OBJECT_ID
            // TODO: add support for $PROPERTY_SET
            NtfsAttribute::ReparsePoint { reparse_point } => {
                let reparse_tag: u32 = reparse_point.get_reparse_tag();
                println!("    Reparse tag\t\t\t\t: 0x{:08x}", reparse_tag);
                // TODO: print tag name

                println!("");
            }
            // TODO: add support for $SECURITY_DESCRIPTOR
            NtfsAttribute::StandardInformation {
                standard_information,
            } => {
                let date_time_string: String =
                    get_date_time_string(&standard_information.creation_time)?;
                println!("    Creation time\t\t\t: {}", date_time_string);

                let date_time_string: String =
                    get_date_time_string(&standard_information.modification_time)?;
                println!("    Modification time\t\t\t: {}", date_time_string);

                let date_time_string: String =
                    get_date_time_string(&standard_information.access_time)?;
                println!("    Access time\t\t\t\t: {}", date_time_string);

                let date_time_string: String =
                    get_date_time_string(&standard_information.entry_modification_time)?;
                println!("    Entry modification time\t\t: {}", date_time_string);

                println!(
                    "    File attribute flags\t\t: 0x{:08x}",
                    standard_information.file_attribute_flags
                );
                let flags_strings: Vec<String> =
                    get_file_attribute_flags_strings(standard_information.file_attribute_flags);
                println!(
                    "{}",
                    flags_strings
                        .iter()
                        .map(|string| format!("        {}", string))
                        .collect::<Vec<String>>()
                        .join("\n")
                );
                if standard_information.file_attribute_flags != 0 {
                    println!("");
                }
            }
            NtfsAttribute::VolumeInformation { volume_information } => {
                println!(
                    "    Format version\t\t\t: {}.{}",
                    volume_information.major_format_version,
                    volume_information.minor_format_version
                );
                println!(
                    "    Volume flags\t\t\t: 0x{:04x}",
                    volume_information.volume_flags
                );
                println!("");
            }
            NtfsAttribute::VolumeName { volume_name } => {
                println!("    Volume name\t\t\t\t: {}", volume_name.to_string());
                println!("");
            }
        }
        Ok(())
    }

    /// Prints information about a file entry.
    fn print_file_entry(file_entry: &mut NtfsFileEntry) -> Result<(), ErrorTrace> {
        // Note that the directory entry file reference can be differrent
        // from the values in the MFT entry.
        let file_reference: u64 = file_entry.get_file_reference();
        println!(
            "    File reference\t\t\t: {}-{}",
            file_reference & 0x0000ffffffffffff,
            file_reference >> 48,
        );
        match file_entry.get_name() {
            Some(name) => println!("    Name\t\t\t\t: {}", name.to_string()),
            None => {}
        };
        println!("    Size\t\t\t\t: {} bytes", file_entry.get_size());

        match file_entry.get_creation_time() {
            Some(date_time) => {
                let date_time_string: String = get_date_time_string(date_time)?;
                println!("    Creation time\t\t\t: {}", date_time_string);
            }
            None => {}
        };
        match file_entry.get_modification_time() {
            Some(date_time) => {
                let date_time_string: String = get_date_time_string(date_time)?;
                println!("    Modification time\t\t\t: {}", date_time_string);
            }
            None => {}
        };
        match file_entry.get_access_time() {
            Some(date_time) => {
                let date_time_string: String = get_date_time_string(date_time)?;
                println!("    Access time\t\t\t\t: {}", date_time_string);
            }
            None => {}
        };
        match file_entry.get_change_time() {
            Some(date_time) => {
                let date_time_string: String = get_date_time_string(date_time)?;
                println!("    Entry modification time\t\t: {}", date_time_string);
            }
            None => {}
        };
        let file_attribute_flags: u32 = file_entry.get_file_attribute_flags();
        println!(
            "    File attribute flags\t\t: 0x{:08x}",
            file_attribute_flags
        );
        let flags_strings: Vec<String> = get_file_attribute_flags_strings(file_attribute_flags);
        println!(
            "{}",
            flags_strings
                .iter()
                .map(|string| format!("        {}", string))
                .collect::<Vec<String>>()
                .join("\n")
        );
        println!("");

        // TODO: print information about reparse point

        Ok(())
    }

    /// Prints information about a specific file entry.
    pub fn print_file_entry_by_identifier(
        data_stream: &DataStreamReference,
        ntfs_entry_identifier: u64,
    ) -> ExitCode {
        let mut ntfs_file_system = NtfsFileSystem::new();

        match ntfs_file_system.read_data_stream(data_stream) {
            Ok(_) => {}
            Err(error) => {
                println!("Unable to open NTFS file system with error: {}", error);
                return ExitCode::FAILURE;
            }
        };
        if ntfs_entry_identifier > u32::MAX as u64 {
            println!(
                "Invalid MFT entry number: {} value out of bounds",
                ntfs_entry_identifier
            );
            return ExitCode::FAILURE;
        }
        let file_entry: NtfsFileEntry =
            match ntfs_file_system.get_file_entry_by_identifier(ntfs_entry_identifier) {
                Ok(file_entry) => file_entry,
                Err(error) => {
                    println!(
                        "Unable to retrive NTFS MFT entry: {} with error: {}",
                        ntfs_entry_identifier, error
                    );
                    return ExitCode::FAILURE;
                }
            };
        println!(
            "New Technologies File System (NTFS) MFT entry: {} information:",
            ntfs_entry_identifier
        );

        if file_entry.is_empty() {
            println!("    Is empty");
        } else {
            println!("    Is allocated\t\t\t: {}", file_entry.is_allocated());

            println!(
                "    File reference\t\t\t: {}-{}",
                file_entry.mft_entry_number, file_entry.sequence_number
            );
            let base_record_file_reference: u64 = file_entry.get_base_record_file_reference();
            if base_record_file_reference == 0 {
                println!("    Base record file reference\t\t: Not set (0)");
            } else {
                println!(
                    "    Base record file reference\t\t: {}-{}",
                    base_record_file_reference >> 48,
                    base_record_file_reference & 0x0000ffffffffffff,
                );
            }
            println!(
                "    Journal sequence number\t\t: {}",
                file_entry.get_journal_sequence_number()
            );

            let number_of_attributes: usize = file_entry.get_number_of_attributes();

            // TODO: print is corrupted.
            println!("");

            for attribute_index in 0..number_of_attributes {
                let attribute: NtfsAttribute =
                    match file_entry.get_attribute_by_index(attribute_index) {
                        Ok(attribute) => attribute,
                        Err(error) => {
                            println!(
                                "Unable to retrive NTFS MFT entry: {} attribute: {} with error: {}",
                                ntfs_entry_identifier, attribute_index, error
                            );
                            return ExitCode::FAILURE;
                        }
                    };
                match Self::print_attribute(&attribute) {
                    Ok(_) => {}
                    Err(error) => {
                        println!("{}", error);
                        return ExitCode::FAILURE;
                    }
                };
            }
        }
        ExitCode::SUCCESS
    }

    /// Prints information about a specific file entry.
    pub fn print_file_entry_by_path(
        data_stream: &DataStreamReference,
        path_components: &[&str],
    ) -> ExitCode {
        let mut ntfs_file_system = NtfsFileSystem::new();

        match ntfs_file_system.read_data_stream(data_stream) {
            Ok(_) => {}
            Err(error) => {
                println!("Unable to open NTFS file system with error: {}", error);
                return ExitCode::FAILURE;
            }
        };
        let ntfs_path: NtfsPath = NtfsPath::from(path_components);
        let mut file_entry: Option<NtfsFileEntry> =
            match ntfs_file_system.get_file_entry_by_path(&ntfs_path) {
                Ok(file_entry) => file_entry,
                Err(error) => {
                    println!("Unable to retrive NTFS file entry with error: {}", error);
                    return ExitCode::FAILURE;
                }
            };
        if file_entry.is_none() {
            println!("No such NTFS file entry");
            return ExitCode::FAILURE;
        }
        println!("New Technologies File System (NTFS) file entry information:");

        match Self::print_file_entry(file_entry.as_mut().unwrap()) {
            Ok(_) => {}
            Err(error) => {
                println!("{}", error);
                return ExitCode::FAILURE;
            }
        };
        ExitCode::SUCCESS
    }

    /// Prints information about the file system.
    pub fn print_file_system(data_stream: &DataStreamReference) -> ExitCode {
        let mut ntfs_file_system = NtfsFileSystem::new();

        match ntfs_file_system.read_data_stream(data_stream) {
            Ok(_) => {}
            Err(error) => {
                println!("Unable to open NTFS file system with error: {}", error);
                return ExitCode::FAILURE;
            }
        }
        println!("New Technologies File System (NTFS) information:");

        match ntfs_file_system.get_format_version() {
            Some((major_version, minor_version)) => {
                println!(
                    "    Format version\t\t\t: {}.{}",
                    major_version, minor_version
                );
            }
            None => {}
        }
        match ntfs_file_system.get_volume_label() {
            Some(volume_label) => {
                println!("    Volume label\t\t\t: {}", volume_label.to_string());
            }
            None => {}
        }
        println!(
            "    Volume serial number\t\t: 0x{:x}",
            ntfs_file_system.volume_serial_number,
        );
        match ntfs_file_system.get_volume_flags() {
            Some(volume_flags) => {
                println!("    Volume flags\t\t\t: 0x{:04x}", volume_flags);
                let flags_strings: Vec<String> = get_ntfs_volume_flags_strings(volume_flags);
                println!(
                    "{}",
                    flags_strings
                        .iter()
                        .map(|string| format!("        {}", string))
                        .collect::<Vec<String>>()
                        .join("\n")
                );
            }
            None => {}
        }
        println!(
            "    Bytes per sector\t\t\t: {} bytes",
            ntfs_file_system.bytes_per_sector
        );
        println!(
            "    Cluster block size\t\t\t: {} bytes",
            ntfs_file_system.cluster_block_size
        );
        println!(
            "    MFT entry size\t\t\t: {} bytes",
            ntfs_file_system.mft_entry_size
        );
        println!(
            "    Index entry size\t\t\t: {} bytes",
            ntfs_file_system.index_entry_size
        );
        println!("");

        ExitCode::SUCCESS
    }

    /// Prints the file system hierarchy.
    pub fn print_hierarchy(data_stream: &DataStreamReference) -> ExitCode {
        let mut ntfs_file_system = NtfsFileSystem::new();

        match ntfs_file_system.read_data_stream(data_stream) {
            Ok(_) => {}
            Err(error) => {
                println!("Unable to open NTFS file system with error: {}", error);
                return ExitCode::FAILURE;
            }
        };
        println!("New Technologies File System (NTFS) hierarchy:");

        let mut file_entry: NtfsFileEntry = match ntfs_file_system.get_root_directory() {
            Ok(file_entry) => file_entry,
            Err(error) => {
                println!(
                    "Unable to retrieve NTFS root directory with error: {}",
                    error
                );
                return ExitCode::FAILURE;
            }
        };
        let mut path_components: Vec<String> = Vec::new();

        match Self::print_hierarchy_file_entry(&mut file_entry, &mut path_components) {
            Ok(_) => {}
            Err(error) => {
                println!("{}", error);
                return ExitCode::FAILURE;
            }
        }
        ExitCode::SUCCESS
    }

    /// Prints the file entry hierarchy.
    fn print_hierarchy_file_entry(
        file_entry: &mut NtfsFileEntry,
        path_components: &mut Vec<String>,
    ) -> Result<(), ErrorTrace> {
        match Self::print_hierarchy_file_entry_as_path(file_entry, path_components) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to print file entry path");
                return Err(error);
            }
        }
        let number_of_file_entries: usize = match file_entry.get_number_of_sub_file_entries() {
            Ok(number_of_file_entries) => number_of_file_entries,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to retrieve number of sub file entries"
                );
                return Err(error);
            }
        };
        for sub_file_entry_index in 0..number_of_file_entries {
            let mut sub_file_entry: NtfsFileEntry =
                match file_entry.get_sub_file_entry_by_index(sub_file_entry_index) {
                    Ok(sub_file_entry) => sub_file_entry,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!(
                                "Unable to retrieve sub file entry: {}",
                                sub_file_entry_index
                            )
                        );
                        return Err(error);
                    }
                };
            match Self::print_hierarchy_file_entry(&mut sub_file_entry, path_components) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to print hierarchy of sub file entry: {}",
                            sub_file_entry_index
                        )
                    );

                    return Err(error);
                }
            }
        }
        if !file_entry.is_root_directory() {
            path_components.pop();
        }
        Ok(())
    }

    /// Prints a file entry as path.
    fn print_hierarchy_file_entry_as_path(
        file_entry: &NtfsFileEntry,
        path_components: &mut Vec<String>,
    ) -> Result<(), ErrorTrace> {
        let path: String = if file_entry.is_root_directory() {
            String::from("/")
        } else {
            let name_string: String = match file_entry.get_name() {
                Some(name) => name.to_string(),
                None => String::new(),
            };
            path_components.push(name_string);
            format!("/{}", path_components.join("/"))
        };
        let number_of_data_forks: usize = match file_entry.get_number_of_data_forks() {
            Ok(number_of_data_forks) => number_of_data_forks,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to retrieve number of data forks"
                );
                return Err(error);
            }
        };
        for data_fork_index in 0..number_of_data_forks {
            let data_fork: NtfsDataFork = match file_entry.get_data_fork_by_index(data_fork_index) {
                Ok(number_of_data_forks) => number_of_data_forks,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to retrieve data fork: {}", data_fork_index)
                    );
                    return Err(error);
                }
            };
            match data_fork.get_name() {
                Some(name) => println!("{}:{}", path, name.to_string()),
                None => println!("{}", path),
            };
        }
        // TODO: print index names?

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_datetime::Filetime;

    #[test]
    fn test_get_date_time_string() -> Result<(), ErrorTrace> {
        let date_time: DateTime = DateTime::Filetime(Filetime::new(0x01cb3a623d0a17ce));
        let timestamp: String = get_date_time_string(&date_time)?;
        assert_eq!(timestamp, "2010-08-12T21:06:31.5468750");

        let date_time: DateTime = DateTime::NotSet;
        let timestamp: String = get_date_time_string(&date_time)?;
        assert_eq!(timestamp, "Not set (0)");

        Ok(())
    }

    #[test]
    fn test_get_file_attribute_flags_strings() {
        let flags_strings: Vec<String> = get_file_attribute_flags_strings(0x00000001);
        assert_eq!(
            flags_strings,
            ["0x00000001: Is read-only (FILE_ATTRIBUTE_READ_ONLY)"]
        );
    }

    #[test]
    fn test_get_ntfs_volume_flags_strings() {
        let flags_strings: Vec<String> = get_ntfs_volume_flags_strings(0x0001);
        assert_eq!(flags_strings, ["0x0001: Is dirty (VOLUME_IS_DIRTY)"],);
    }

    // TODO: add tests for print_attribute
    // TODO: add tests for print_file_entry
    // TODO: add tests for print_file_entry_by_identifier
    // TODO: add tests for print_file_entry_by_path
    // TODO: add tests for print_file_system
    // TODO: add tests for print_hierarchy
    // TODO: add tests for print_hierarchy_file_entry
    // TODO: add tests for print_hierarchy_file_entry_as_path
}
