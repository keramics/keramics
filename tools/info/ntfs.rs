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
use std::io;
use std::process::ExitCode;
use std::sync::Arc;

use keramics::datetime::DateTime;
use keramics::formats::ntfs::constants::*;
use keramics::formats::ntfs::{
    NtfsAttribute, NtfsDataFork, NtfsFileEntry, NtfsFileSystem, NtfsPath,
};
use keramics::vfs::{VfsDataStreamReference, VfsFileSystem, VfsPath};

use super::bodyfile;

pub const FILE_ATTRIBUTE_FLAG_READ_ONLY: u32 = 0x00000001;
pub const FILE_ATTRIBUTE_FLAG_SYSTEM: u32 = 0x00000004;

/// Retrieves the string representation of a date and time value.
fn get_date_time_string(date_time: &DateTime) -> io::Result<String> {
    match date_time {
        DateTime::NotSet => Ok("Not set (0)".to_string()),
        DateTime::Filetime(filetime) => Ok(filetime.to_iso8601_string()),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Unsupported date time"),
        )),
    }
}

/// Retrieves a file mode string representation of file attribute flags.
fn get_file_mode_string(file_attribute_flags: u32) -> String {
    let mut string_parts: Vec<&str> = vec!["-", "r", "w", "x", "r", "w", "x", "r", "w", "x"];

    // TODO: determine file (entry) type ("d", "l", "-")

    if file_attribute_flags & FILE_ATTRIBUTE_FLAG_READ_ONLY != 0
        || file_attribute_flags & FILE_ATTRIBUTE_FLAG_SYSTEM != 0
    {
        string_parts[2] = "-";
        string_parts[5] = "-";
        string_parts[8] = "-";
    }
    string_parts.join("")
}

/// Prints the file attribute flags.
fn print_attribute_flags(flags: u32) {
    if flags & 0x00000001 != 0 {
        println!("        0x00000001: Is read-only (FILE_ATTRIBUTE_READ_ONLY)");
    }
    if flags & 0x00000002 != 0 {
        println!("        0x00000002: Is hidden (FILE_ATTRIBUTE_HIDDEN)");
    }
    if flags & 0x00000004 != 0 {
        println!("        0x00000004: Is system (FILE_ATTRIBUTE_SYSTEM)");
    }

    if flags & 0x00000010 != 0 {
        println!("        0x00000010: Is directory (FILE_ATTRIBUTE_DIRECTORY)");
    }
    if flags & 0x00000020 != 0 {
        println!("        0x00000020: Should be archived (FILE_ATTRIBUTE_ARCHIVE)");
    }
    if flags & 0x00000040 != 0 {
        println!("        0x00000040: Is device (FILE_ATTRIBUTE_DEVICE)");
    }
    if flags & 0x00000080 != 0 {
        println!("        0x00000080: Is normal (FILE_ATTRIBUTE_NORMAL)");
    }
    if flags & 0x00000100 != 0 {
        println!("        0x00000100: Is temporary (FILE_ATTRIBUTE_TEMPORARY)");
    }
    if flags & 0x00000200 != 0 {
        println!("        0x00000200: Is a sparse file (FILE_ATTRIBUTE_SPARSE_FILE)");
    }
    if flags & 0x00000400 != 0 {
        println!("        0x00000400: Is a reparse point or symbolic link (FILE_ATTRIBUTE_FLAG_REPARSE_POINT)");
    }
    if flags & 0x00000800 != 0 {
        println!("        0x00000800: Is compressed (FILE_ATTRIBUTE_COMPRESSED)");
    }
    if flags & 0x00001000 != 0 {
        println!("        0x00001000: Is offline (FILE_ATTRIBUTE_OFFLINE)");
    }
    if flags & 0x00002000 != 0 {
        println!("        0x00002000: Content should not be indexed (FILE_ATTRIBUTE_NOT_CONTENT_INDEXED)");
    }
    if flags & 0x00004000 != 0 {
        println!("        0x00004000: Is encrypted (FILE_ATTRIBUTE_ENCRYPTED)");
    }

    if flags & 0x00010000 != 0 {
        println!("        0x00010000: Is virtual (FILE_ATTRIBUTE_VIRTUAL)");
    }

    if flags & 0x20000000 != 0 {
        println!("        0x20000000: UNKNOWN (Is index view)");
    }
    println!("");
}

/// Prints the volume flags.
fn print_ntfs_volume_flags(flags: u16) {
    if flags & 0x0001 != 0 {
        println!("        0x0001: Is dirty (VOLUME_IS_DIRTY)");
    }
    if flags & 0x0002 != 0 {
        println!("        0x0002: Re-size journal ($LogFile) (VOLUME_RESIZE_LOG_FILE)");
    }
    if flags & 0x0004 != 0 {
        println!("        0x0004: Mounted on Windows NT 4 (VOLUME_MOUNTED_ON_NT4)");
    }
    if flags & 0x0008 != 0 {
        println!("        0x0008: Is dirty (VOLUME_IS_DIRTY)");
    }
    if flags & 0x0010 != 0 {
        println!("        0x0010: Delete USN in progress (VOLUME_DELETE_USN_UNDERWAY)");
    }
    if flags & 0x0020 != 0 {
        println!("        0x0020: Repair object identifiers (VOLUME_REPAIR_OBJECT_ID)");
    }

    if flags & 0x4000 != 0 {
        println!("        0x4000: chkdsk in progress (VOLUME_CHKDSK_UNDERWAY)");
    }
    if flags & 0x8000 != 0 {
        println!("        0x8000: Modified by chkdsk (VOLUME_MODIFIED_BY_CHKDSK)");
    }
    println!("");
}

/// Prints information about a New Technologies File System (NTFS).
pub fn print_ntfs_file_system(
    vfs_file_system: &Arc<VfsFileSystem>,
    vfs_path: &VfsPath,
) -> ExitCode {
    let mut ntfs_file_system = NtfsFileSystem::new();

    match ntfs_file_system.open(vfs_file_system, vfs_path) {
        Ok(_) => {}
        Err(error) => {
            println!("Unable to open NTFS file system with error: {}", error);
            return ExitCode::FAILURE;
        }
    };
    println!("New Technologies File System (NTFS) information:");

    match ntfs_file_system.get_format_version() {
        Some((major_version, minor_version)) => {
            println!(
                "    Format version\t\t\t: {}.{}",
                major_version, minor_version
            );
        }
        None => {}
    };
    match ntfs_file_system.get_volume_label() {
        Some(volume_label) => {
            println!("    Volume label\t\t\t: {}", volume_label.to_string());
        }
        None => {}
    };
    println!(
        "    Volume serial number\t\t: 0x{:x}",
        ntfs_file_system.volume_serial_number,
    );
    match ntfs_file_system.get_volume_flags() {
        Some(volume_flags) => {
            println!("    Volume flags\t\t\t: 0x{:04x}", volume_flags);
            print_ntfs_volume_flags(volume_flags);
        }
        None => {}
    };
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

/// Prints information about a specific entry of a New Technologies File System (NTFS).
pub fn print_entry_ntfs_file_system(
    vfs_file_system: &Arc<VfsFileSystem>,
    vfs_path: &VfsPath,
    ntfs_entry_identifier: u64,
) -> ExitCode {
    let mut ntfs_file_system = NtfsFileSystem::new();

    match ntfs_file_system.open(vfs_file_system, vfs_path) {
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
    let mut file_entry: NtfsFileEntry =
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
        let (base_record_mft_entry, base_record_sequence_number): (u64, u16) =
            file_entry.get_base_record_file_reference();
        if base_record_mft_entry == 0 {
            println!("    Base record file reference\t\t: Not set (0)");
        } else {
            println!(
                "    Base record file reference\t\t: {}-{}",
                base_record_mft_entry, base_record_sequence_number
            );
        }
        println!(
            "    Journal sequence number\t\t: {}",
            file_entry.get_journal_sequence_number()
        );

        let number_of_attributes: usize = match file_entry.get_number_of_attributes() {
            Ok(number_of_attributes) => number_of_attributes,
            Err(error) => {
                println!(
                    "Unable to retrive NTFS MFT entry: {} number of attributes with error: {}",
                    ntfs_entry_identifier, error
                );
                return ExitCode::FAILURE;
            }
        };
        println!("    Number of attributes\t\t: {}", number_of_attributes);

        // TODO: print is corrupted.
        println!("");

        for attribute_index in 0..number_of_attributes {
            let mut attribute: NtfsAttribute =
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
            println!("Attribute: {}", attribute_index + 1);

            match print_ntfs_attribute(&mut attribute) {
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

/// Prints information about a New Technologies File System (NTFS) attribute.
fn print_ntfs_attribute(attribute: &mut NtfsAttribute) -> io::Result<()> {
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
            "    Attribute type\t\t\t: {} (0x{:08x})",
            attribute_type_string, attribute_type
        ),
        None => println!("    Attribute type\t\t\t: 0x{:08x}", attribute_type),
    };
    match attribute {
        // TODO: add support for $ATTRIBUTE_LIST
        // TODO: add support for $DATA
        // TODO: add support for $BITMAP
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
            print_attribute_flags(file_name.file_attribute_flags);
        }
        // TODO: add support for $INDEX_ALLOCATION
        // TODO: add support for $INDEX_ROOT
        // TODO: add support for $LOGGED_UTILITY_STREAM
        // TODO: add support for $OBJECT_ID
        // TODO: add support for $PROPERTY_SET
        // TODO: add support for $REPARSE_POINT
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

            let date_time_string: String = get_date_time_string(&standard_information.access_time)?;
            println!("    Access time\t\t\t\t: {}", date_time_string);

            let date_time_string: String =
                get_date_time_string(&standard_information.entry_modification_time)?;
            println!("    Entry modification time\t\t: {}", date_time_string);

            println!(
                "    File attribute flags\t\t: 0x{:08x}",
                standard_information.file_attribute_flags
            );
            print_attribute_flags(standard_information.file_attribute_flags);
        }
        NtfsAttribute::VolumeInformation { volume_information } => {
            println!(
                "    Format version\t\t\t: {}.{}",
                volume_information.major_format_version, volume_information.minor_format_version
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
        _ => {
            println!("");
        }
    };
    Ok(())
}

/// Prints the hierarchy of a New Technologies File System (NTFS).
pub fn print_hierarcy_ntfs_file_system(
    vfs_file_system: &Arc<VfsFileSystem>,
    vfs_path: &VfsPath,
    in_bodyfile_format: bool,
) -> ExitCode {
    let mut ntfs_file_system = NtfsFileSystem::new();

    match ntfs_file_system.open(vfs_file_system, vfs_path) {
        Ok(_) => {}
        Err(error) => {
            println!("Unable to open NTFS file system with error: {}", error);
            return ExitCode::FAILURE;
        }
    };
    if in_bodyfile_format {
        println!("{}", bodyfile::BODYFILE_HEADER);
    } else {
        println!("New Technologies File System (NTFS) hierarchy:");
    }
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
    match print_hierarcy_ntfs_file_entry(&mut file_entry, &mut path_components, in_bodyfile_format)
    {
        Ok(_) => {}
        Err(error) => {
            println!("{}", error);
            return ExitCode::FAILURE;
        }
    };
    ExitCode::SUCCESS
}

/// Prints information about a New Technologies File System (NTFS) file entry.
fn print_ntfs_file_entry(file_entry: &mut NtfsFileEntry) -> io::Result<()> {
    println!(
        "    File reference\t\t\t: {}-{}",
        file_entry.mft_entry_number, file_entry.sequence_number
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
            println!("    Access time\t\t\t: {}", date_time_string);
        }
        None => {}
    };
    match file_entry.get_change_time() {
        Some(date_time) => {
            let date_time_string: String = get_date_time_string(date_time)?;
            println!("    Entry modification time\t\t\t: {}", date_time_string);
        }
        None => {}
    };
    let file_attribute_flags: u32 = file_entry.get_file_attribute_flags();
    println!(
        "    File attribute flags\t\t: 0x{:08x}",
        file_attribute_flags
    );
    print_attribute_flags(file_attribute_flags);

    println!("");

    Ok(())
}

/// Prints information about a New Technologies File System (NTFS) file entry in bodyfile format.
fn print_ntfs_file_entry_bodyfile(
    file_entry: &mut NtfsFileEntry,
    path_components: &mut Vec<String>,
) -> io::Result<()> {
    let file_attribute_flags: u32 = file_entry.get_file_attribute_flags();

    let path: String = if file_entry.mft_entry_number == 5 {
        String::from("/")
    } else {
        let name_string: String = match file_entry.get_name() {
            Some(name) => name.to_string(),
            None => String::new(),
        };
        path_components.push(name_string);
        format!("/{}", path_components.join("/"))
    };
    // TODO: add suport for symbolic link path suffix.
    let path_suffix: String = String::new();

    let file_mode_string: String = get_file_mode_string(file_attribute_flags);

    // TODO: add support for owner_identifier.
    // TODO: add support for group_identifier.
    let size: u64 = file_entry.get_size();

    let access_time: String = bodyfile::format_as_timestamp(file_entry.get_access_time())?;
    let modification_time: String =
        bodyfile::format_as_timestamp(file_entry.get_modification_time())?;
    let change_time: String = bodyfile::format_as_timestamp(file_entry.get_change_time())?;
    let creation_time: String = bodyfile::format_as_timestamp(file_entry.get_creation_time())?;

    let number_of_data_forks: usize = file_entry.get_number_of_data_forks()?;

    if number_of_data_forks == 0 {
        // TODO: have flag control calculate md5
        // String::from("0")
        let md5: String = String::from("00000000000000000000000000000000");

        println!(
            "{}|{}{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
            md5,
            path,
            path_suffix,
            file_entry.mft_entry_number,
            file_mode_string,
            0,
            0,
            size,
            access_time,
            modification_time,
            change_time,
            creation_time
        );
    } else {
        for data_fork in 0..number_of_data_forks {
            let data_fork: NtfsDataFork = file_entry.get_data_fork_by_index(data_fork)?;

            // TODO: have flag control calculate md5
            // String::from("0")
            let data_stream: VfsDataStreamReference = data_fork.get_data_stream()?;

            let md5: String = match data_stream.with_write_lock() {
                Ok(mut data_stream) => bodyfile::calculate_md5(&mut data_stream)?,
                Err(error) => return Err(keramics::error_to_io_error!(error)),
            };
            let data_fork_name: String = match data_fork.get_name() {
                Some(name) => format!(":{}", name.to_string()),
                None => String::new(),
            };
            println!(
                "{}|{}{}{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
                md5,
                path,
                data_fork_name,
                path_suffix,
                file_entry.mft_entry_number,
                file_mode_string,
                0,
                0,
                size,
                access_time,
                modification_time,
                change_time,
                creation_time
            );
        }
    }
    // TODO: print $FILE_NAME attribute
    // TODO: print index names

    Ok(())
}

/// Prints path of a New Technologies File System (NTFS) file entry.
fn print_ntfs_file_entry_path(
    file_entry: &NtfsFileEntry,
    path_components: &mut Vec<String>,
) -> io::Result<()> {
    let path: String = if file_entry.mft_entry_number == 5 {
        String::from("/")
    } else {
        let name_string: String = match file_entry.get_name() {
            Some(name) => name.to_string(),
            None => String::new(),
        };
        path_components.push(name_string);
        format!("/{}", path_components.join("/"))
    };
    let number_of_data_forks: usize = file_entry.get_number_of_data_forks()?;

    for data_fork in 0..number_of_data_forks {
        let data_fork: NtfsDataFork = file_entry.get_data_fork_by_index(data_fork)?;
        match data_fork.get_name() {
            Some(name) => println!("{}:{}", path, name.to_string()),
            None => println!("{}", path),
        };
    }
    // TODO: print index names?

    Ok(())
}

/// Prints the hierarchy of a New Technologies File System (NTFS) file entry.
fn print_hierarcy_ntfs_file_entry(
    file_entry: &mut NtfsFileEntry,
    path_components: &mut Vec<String>,
    in_bodyfile_format: bool,
) -> io::Result<()> {
    if in_bodyfile_format {
        print_ntfs_file_entry_bodyfile(file_entry, path_components)?;
    } else {
        print_ntfs_file_entry_path(file_entry, path_components)?;
    }
    let number_of_file_entries: usize = file_entry.get_number_of_sub_file_entries()?;
    for sub_file_entry_index in 0..number_of_file_entries {
        let mut sub_file_entry: NtfsFileEntry =
            file_entry.get_sub_file_entry_by_index(sub_file_entry_index)?;

        print_hierarcy_ntfs_file_entry(&mut sub_file_entry, path_components, in_bodyfile_format)?;
    }
    if file_entry.mft_entry_number != 5 {
        path_components.pop();
    }
    Ok(())
}

/// Prints information about a specific path of a New Technologies File System (NTFS).
pub fn print_path_ntfs_file_system(
    vfs_file_system: &Arc<VfsFileSystem>,
    vfs_path: &VfsPath,
    path: &String,
) -> ExitCode {
    let mut ext_file_system = NtfsFileSystem::new();

    match ext_file_system.open(vfs_file_system, vfs_path) {
        Ok(_) => {}
        Err(error) => {
            println!("Unable to open NTFS file system with error: {}", error);
            return ExitCode::FAILURE;
        }
    };
    let ext_path: NtfsPath = NtfsPath::from(path);
    let mut file_entry: Option<NtfsFileEntry> =
        match ext_file_system.get_file_entry_by_path(&ext_path) {
            Ok(file_entry) => file_entry,
            Err(error) => {
                println!(
                    "Unable to retrive NTFS file entry: {} with error: {}",
                    path, error
                );
                return ExitCode::FAILURE;
            }
        };
    if file_entry.is_none() {
        println!("No such to NTFS file entry: {}", path);
        return ExitCode::FAILURE;
    }
    println!("New Technologies File System (NTFS) file entry information:");

    match print_ntfs_file_entry(file_entry.as_mut().unwrap()) {
        Ok(_) => {}
        Err(error) => {
            println!("{}", error);
            return ExitCode::FAILURE;
        }
    };
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: add tests for get_date_time_string

    #[test]
    fn test_get_file_mode_string() {
        let string: String = get_file_mode_string(0x00000020);
        assert_eq!(string, "-rwxrwxrwx");

        let string: String = get_file_mode_string(0x00000006);
        assert_eq!(string, "-r-xr-xr-x");
    }

    // TODO: add tests for print_attribute_flags
    // TODO: add tests for print_ntfs_volume_flags
    // TODO: add tests for print_ntfs_file_system
    // TODO: add tests for print_entry_ntfs_file_system
    // TODO: add tests for print_hierarcy_ntfs_file_system
    // TODO: add tests for print_ntfs_file_entry
    // TODO: add tests for print_ntfs_file_entry_bodyfile
    // TODO: add tests for print_ntfs_file_entry_path
    // TODO: add tests for print_hierarcy_ntfs_file_entry
    // TODO: add tests for print_path_ntfs_file_system
}
