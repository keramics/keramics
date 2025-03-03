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

use std::io;
use std::process::ExitCode;
use std::sync::Arc;

use keramics::datetime::DateTime;
use keramics::formats::ext::constants::*;
use keramics::formats::ext::{ExtFileEntry, ExtFileSystem, ExtPath};
use keramics::types::ByteString;
use keramics::vfs::{VfsDataStreamReference, VfsFileSystem, VfsPath};

use super::bodyfile;

/// Retrieves the string representation of a date and time value.
fn get_date_time_string(date_time: &DateTime) -> io::Result<String> {
    match date_time {
        DateTime::NotSet => Ok("Not set (0)".to_string()),
        DateTime::PosixTime32(posix_time32) => Ok(posix_time32.to_iso8601_string()),
        DateTime::PosixTime64Ns(posix_time64ns) => Ok(posix_time64ns.to_iso8601_string()),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Unsupported date time"),
        )),
    }
}

/// Retrieves the string representation of a file mode.
fn get_file_mode_string(file_mode: u16) -> String {
    let mut string_parts: Vec<&str> = vec!["-"; 10];

    if file_mode & 0x0001 != 0 {
        string_parts[9] = "x";
    }
    if file_mode & 0x0002 != 0 {
        string_parts[8] = "w";
    }
    if file_mode & 0x0004 != 0 {
        string_parts[7] = "r";
    }
    if file_mode & 0x0008 != 0 {
        string_parts[6] = "x";
    }
    if file_mode & 0x0010 != 0 {
        string_parts[5] = "w";
    }
    if file_mode & 0x0020 != 0 {
        string_parts[4] = "r";
    }
    if file_mode & 0x0040 != 0 {
        string_parts[3] = "x";
    }
    if file_mode & 0x0080 != 0 {
        string_parts[2] = "w";
    }
    if file_mode & 0x0100 != 0 {
        string_parts[1] = "r";
    }
    string_parts[0] = match file_mode & 0xf000 {
        EXT_FILE_MODE_TYPE_FIFO => "p",
        EXT_FILE_MODE_TYPE_CHARACTER_DEVICE => "c",
        EXT_FILE_MODE_TYPE_DIRECTORY => "d",
        EXT_FILE_MODE_TYPE_BLOCK_DEVICE => "b",
        EXT_FILE_MODE_TYPE_SYMBOLIC_LINK => "l",
        EXT_FILE_MODE_TYPE_SOCKET => "s",
        _ => "-",
    };
    string_parts.join("")
}

/// Prints the compatible feature flags.
fn print_ext_compatible_feature_flags(flags: u32) {
    if flags & 0x00000001 != 0 {
        println!("        0x00000001: Pre-allocate directory blocks (EXT2_COMPAT_PREALLOC)");
    }
    if flags & 0x00000002 != 0 {
        println!("        0x00000002: Has AFS server inodes (EXT2_FEATURE_COMPAT_IMAGIC_INODES)");
    }
    if flags & 0x00000004 != 0 {
        println!("        0x00000004: Has journal (EXT3_FEATURE_COMPAT_HAS_JOURNAL)");
    }
    if flags & 0x00000008 != 0 {
        println!("        0x00000008: Has extended attributes (EXT2_FEATURE_COMPAT_EXT_ATTR)");
    }
    if flags & 0x00000010 != 0 {
        println!("        0x00000010: Is resizable (EXT2_FEATURE_COMPAT_RESIZE_INO)");
    }
    if flags & 0x00000020 != 0 {
        println!("        0x00000020: Has indexed directories (EXT2_FEATURE_COMPAT_DIR_INDEX)");
    }

    if flags & 0x00000200 != 0 {
        println!("        0x00000200: Has sparse superblock version 2 (EXT4_FEATURE_COMPAT_SPARSE_SUPER2)");
    }
    if flags & 0x00000400 != 0 {
        println!("        0x00000400: (EXT4_FEATURE_COMPAT_FAST_COMMIT)");
    }
    if flags & 0x00000800 != 0 {
        println!("        0x00000800: (EXT4_FEATURE_COMPAT_STABLE_INODES)");
    }
    if flags & 0x00001000 != 0 {
        println!("        0x00001000: Has orphan file (EXT4_FEATURE_COMPAT_ORPHAN_FILE)");
    }
    println!("");
}

/// Prints the incompatible feature flags.
fn print_ext_incompatible_feature_flags(flags: u32) {
    if flags & 0x00000001 != 0 {
        println!("        0x00000001: Has compression (EXT2_FEATURE_INCOMPAT_COMPRESSION)");
    }
    if flags & 0x00000002 != 0 {
        println!(
            "        0x00000002: Directory entry has file type (EXT2_FEATURE_INCOMPAT_FILETYPE)"
        );
    }
    if flags & 0x00000004 != 0 {
        println!("        0x00000004: Needs recovery (EXT3_FEATURE_INCOMPAT_RECOVER)");
    }
    if flags & 0x00000008 != 0 {
        println!("        0x00000008: Has journal device (EXT3_FEATURE_INCOMPAT_JOURNAL_DEV)");
    }
    if flags & 0x00000010 != 0 {
        println!("        0x00000010: Has meta block groups (EXT2_FEATURE_INCOMPAT_META_BG)");
    }

    if flags & 0x00000040 != 0 {
        println!("        0x00000040: Has extents (EXT4_FEATURE_INCOMPAT_EXTENTS)");
    }
    if flags & 0x00000080 != 0 {
        println!("        0x00000080: Has 64-bit support (EXT4_FEATURE_INCOMPAT_64BIT)");
    }
    if flags & 0x00000100 != 0 {
        println!("        0x00000100: (EXT4_FEATURE_INCOMPAT_MMP)");
    }
    if flags & 0x00000200 != 0 {
        println!("        0x00000200: Has flexible block groups (EXT4_FEATURE_INCOMPAT_FLEX_BG)");
    }
    if flags & 0x00000400 != 0 {
        println!("        0x00000400: (EXT4_FEATURE_INCOMPAT_EA_INODE)");
    }

    if flags & 0x00001000 != 0 {
        println!("        0x00001000: (EXT4_FEATURE_INCOMPAT_DIRDATA)");
    }
    if flags & 0x00002000 != 0 {
        println!(
            "        0x00002000: Has metadata checksum seed (EXT4_FEATURE_INCOMPAT_CSUM_SEED)"
        );
    }
    if flags & 0x00004000 != 0 {
        println!("        0x00004000: (EXT4_FEATURE_INCOMPAT_LARGEDIR)");
    }
    if flags & 0x00008000 != 0 {
        println!("        0x00008000: (EXT4_FEATURE_INCOMPAT_INLINE_DATA)");
    }
    if flags & 0x00010000 != 0 {
        println!("        0x00010000: (EXT4_FEATURE_INCOMPAT_ENCRYPT)");
    }
    if flags & 0x00020000 != 0 {
        println!("        0x00020000: (EXT4_FEATURE_INCOMPAT_CASEFOLD)");
    }
    println!("");
}

/// Prints the read-only compatible feature flags.
fn print_ext_read_only_compatible_feature_flags(flags: u32) {
    if flags & 0x00000001 != 0 {
        println!("        0x00000001: Has sparse superblocks and group descriptor tables (EXT2_FEATURE_RO_COMPAT_SPARSE_SUPER)");
    }
    if flags & 0x00000002 != 0 {
        println!("        0x00000002: Contains large files (EXT2_FEATURE_RO_COMPAT_LARGE_FILE)");
    }
    if flags & 0x00000004 != 0 {
        println!("        0x00000004: Has directory B-tree (EXT2_FEATURE_RO_COMPAT_BTREE_DIR)");
    }
    if flags & 0x00000008 != 0 {
        println!("        0x00000008: (EXT4_FEATURE_RO_COMPAT_HUGE_FILE)");
    }
    if flags & 0x00000010 != 0 {
        println!("        0x00000010: (EXT4_FEATURE_RO_COMPAT_GDT_CSUM)");
    }
    if flags & 0x00000020 != 0 {
        println!("        0x00000020: (EXT4_FEATURE_RO_COMPAT_DIR_NLINK)");
    }
    if flags & 0x00000040 != 0 {
        println!("        0x00000040: Has large inodes (EXT4_FEATURE_RO_COMPAT_EXTRA_ISIZE)");
    }
    if flags & 0x00000080 != 0 {
        println!("        0x00000080: (EXT4_FEATURE_RO_COMPAT_HAS_SNAPSHOT)");
    }
    if flags & 0x00000100 != 0 {
        println!("        0x00000100: (EXT4_FEATURE_RO_COMPAT_QUOTA)");
    }
    if flags & 0x00000200 != 0 {
        println!("        0x00000200: (EXT4_FEATURE_RO_COMPAT_BIGALLOC)");
    }
    if flags & 0x00000400 != 0 {
        println!(
            "        0x00000400: Has metadata checksums (EXT4_FEATURE_RO_COMPAT_METADATA_CSUM)"
        );
    }
    if flags & 0x00000800 != 0 {
        println!("        0x00000800: (EXT4_FEATURE_RO_COMPAT_REPLICA)");
    }
    if flags & 0x00001000 != 0 {
        println!("        0x00001000: (EXT4_FEATURE_RO_COMPAT_READONLY)");
    }
    if flags & 0x00002000 != 0 {
        println!("        0x00002000: (EXT4_FEATURE_RO_COMPAT_PROJECT)");
    }
    if flags & 0x00004000 != 0 {
        println!("        0x00004000: (EXT4_FEATURE_RO_COMPAT_SHARED_BLOCKS)");
    }
    if flags & 0x00008000 != 0 {
        println!("        0x00008000: (EXT4_FEATURE_RO_COMPAT_VERITY)");
    }
    if flags & 0x00010000 != 0 {
        println!("        0x00010000: Orphan file may be non-empty (EXT4_FEATURE_RO_COMPAT_ORPHAN_PRESENT)");
    }
    println!("");
}

/// Prints information about an Extended File System (ext).
pub fn print_ext_file_system(vfs_file_system: &Arc<VfsFileSystem>, vfs_path: &VfsPath) -> ExitCode {
    let mut ext_file_system = ExtFileSystem::new();

    match ext_file_system.open(vfs_file_system, vfs_path) {
        Ok(_) => {}
        Err(error) => {
            println!("Unable to open ext file system with error: {}", error);
            return ExitCode::FAILURE;
        }
    };
    println!("Extended File System (ext) information:");

    let format_version: u8 = ext_file_system.get_format_version();
    println!("    Format version\t\t\t: ext{}", format_version);

    let volume_label: &ByteString = ext_file_system.get_volume_label();
    println!("    Volume label\t\t\t: {}", volume_label.to_string());

    let feature_flags: u32 = ext_file_system.get_compatible_feature_flags();
    println!("    Compatible features\t\t\t: 0x{:08x}", feature_flags);
    print_ext_compatible_feature_flags(feature_flags);

    let feature_flags: u32 = ext_file_system.get_incompatible_feature_flags();
    println!("    Incompatible features\t\t: 0x{:08x}", feature_flags);
    print_ext_incompatible_feature_flags(feature_flags);

    let feature_flags: u32 = ext_file_system.get_read_only_compatible_feature_flags();
    println!(
        "    Read-only compatible features\t: 0x{:08x}",
        feature_flags
    );
    print_ext_read_only_compatible_feature_flags(feature_flags);

    println!(
        "    Number of inodes\t\t\t: {}",
        ext_file_system.number_of_inodes
    );
    println!(
        "    Last mount path\t\t\t: {}",
        ext_file_system.last_mount_path.to_string()
    );
    let date_time_string: String = match ext_file_system.last_mount_time {
        DateTime::NotSet => "Not set (0)".to_string(),
        DateTime::PosixTime32(posix_time32) => posix_time32.to_iso8601_string(),
        _ => {
            println!("Unsupported last mount time");
            return ExitCode::FAILURE;
        }
    };
    println!("    Last mount time\t\t\t: {}", date_time_string);
    let date_time_string: String = match ext_file_system.last_written_time {
        DateTime::NotSet => "Not set (0)".to_string(),
        DateTime::PosixTime32(posix_time32) => posix_time32.to_iso8601_string(),
        _ => {
            println!("Unsupported last written time");
            return ExitCode::FAILURE;
        }
    };
    println!("    Last written time\t\t\t: {}", date_time_string);
    println!("");

    ExitCode::SUCCESS
}

/// Prints information about an Extended File System (ext) file entry.
fn print_ext_file_entry(file_entry: &mut ExtFileEntry) -> io::Result<()> {
    println!("    Inode number\t\t\t: {}", file_entry.inode_number);

    match file_entry.get_name() {
        Some(name) => println!("    Name\t\t\t\t: {}", name.to_string()),
        None => {}
    };
    println!("    Size\t\t\t\t: {}", file_entry.get_size());

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
            println!("    Inode change time\t\t\t: {}", date_time_string);
        }
        None => {}
    };
    match file_entry.get_creation_time() {
        Some(date_time) => {
            let date_time_string: String = get_date_time_string(date_time)?;
            println!("    Creation time\t\t\t: {}", date_time_string);
        }
        None => {}
    };
    let date_time: &DateTime = file_entry.get_deletion_time();
    let date_time_string: String = get_date_time_string(date_time)?;
    println!("    Deletion time\t\t\t: {}", date_time_string);

    println!(
        "    Number of links\t\t\t: {}",
        file_entry.get_number_of_links()
    );
    println!(
        "    Owner identifier\t\t\t: {}",
        file_entry.get_owner_identifier()
    );
    println!(
        "    Group identifier\t\t\t: {}",
        file_entry.get_group_identifier()
    );
    let file_mode: u16 = file_entry.get_file_mode();
    let file_mode_string: String = get_file_mode_string(file_mode);

    println!(
        "    File mode\t\t\t\t: {} (0o{:0o})",
        file_mode_string, file_mode
    );
    match file_entry.get_device_identifier()? {
        Some(device_identifier) => {
            println!(
                "    Device number\t\t\t: {},{}",
                device_identifier >> 8,
                device_identifier & 0x00ff
            );
        }
        None => {}
    };
    match file_entry.get_symbolic_link_target()? {
        Some(symbolic_link_target) => {
            println!(
                "    Symbolic link target\t\t: {}",
                symbolic_link_target.to_string()
            );
        }
        None => {}
    };
    let number_of_attributes: usize = file_entry.get_number_of_attributes()?;
    if number_of_attributes > 0 {
        println!("    Extended attributes:");

        // TODO: print extended attribute names.
        // Attribute: 1	: security.selinux
    }
    println!("");

    Ok(())
}

/// Prints information about an Extended File System (ext) file entry in bodyfile format.
fn print_ext_file_entry_bodyfile(
    file_entry: &mut ExtFileEntry,
    path_components: &mut Vec<String>,
) -> io::Result<()> {
    let file_mode: u16 = file_entry.get_file_mode();

    // TODO: have flag control calculate md5
    // String::from("0")
    let result: Option<VfsDataStreamReference> = file_entry.get_data_stream_by_name(None)?;
    let md5: String = match result {
        Some(vfs_data_stream) => {
            let md5_string: String = match vfs_data_stream.with_write_lock() {
                Ok(mut data_stream) => bodyfile::calculate_md5(&mut data_stream)?,
                Err(error) => return Err(keramics::error_to_io_error!(error)),
            };
            md5_string
        }
        None => String::from("00000000000000000000000000000000"),
    };
    let path: String = if file_entry.inode_number == 2 {
        String::from("/")
    } else {
        let name_string: String = match file_entry.get_name() {
            Some(name) => name.to_string(),
            None => String::new(),
        };
        path_components.push(name_string);
        format!("/{}", path_components.join("/"))
    };
    let path_suffix: String = match file_entry.get_symbolic_link_target()? {
        Some(symbolic_link_target) => format!(" -> {}", symbolic_link_target.to_string()),
        None => String::new(),
    };
    let file_mode_string: String = get_file_mode_string(file_mode);

    let owner_identifier: u32 = file_entry.get_owner_identifier();
    let group_identifier: u32 = file_entry.get_group_identifier();
    let size: u64 = file_entry.get_size();

    let access_time: String = bodyfile::format_as_timestamp(file_entry.get_access_time())?;
    let modification_time: String =
        bodyfile::format_as_timestamp(file_entry.get_modification_time())?;
    let change_time: String = bodyfile::format_as_timestamp(file_entry.get_change_time())?;
    let creation_time: String = bodyfile::format_as_timestamp(file_entry.get_creation_time())?;

    println!(
        "{}|{}{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
        md5,
        path,
        path_suffix,
        file_entry.inode_number,
        file_mode_string,
        owner_identifier,
        group_identifier,
        size,
        access_time,
        modification_time,
        change_time,
        creation_time
    );
    Ok(())
}

/// Prints path of an Extended File System (ext) file entry.
fn print_ext_file_entry_path(
    file_entry: &ExtFileEntry,
    path_components: &mut Vec<String>,
) -> io::Result<()> {
    let path: String = if file_entry.inode_number == 2 {
        String::from("/")
    } else {
        let name_string: String = match file_entry.get_name() {
            Some(name) => name.to_string(),
            None => String::new(),
        };
        path_components.push(name_string);
        format!("/{}", path_components.join("/"))
    };
    println!("{}", path);

    Ok(())
}

/// Prints information about a specific entry of an Extended File System (ext).
pub fn print_entry_ext_file_system(
    vfs_file_system: &Arc<VfsFileSystem>,
    vfs_path: &VfsPath,
    ext_entry_identifier: u64,
) -> ExitCode {
    let mut ext_file_system = ExtFileSystem::new();

    match ext_file_system.open(vfs_file_system, vfs_path) {
        Ok(_) => {}
        Err(error) => {
            println!("Unable to open ext file system with error: {}", error);
            return ExitCode::FAILURE;
        }
    };
    if ext_entry_identifier > u32::MAX as u64 {
        println!(
            "Invalid inode number: {} value out of bounds",
            ext_entry_identifier
        );
        return ExitCode::FAILURE;
    }
    let mut file_entry: ExtFileEntry =
        match ext_file_system.get_file_entry_by_identifier(ext_entry_identifier as u32) {
            Ok(file_entry) => file_entry,
            Err(error) => {
                println!(
                    "Unable to retrive ext file entry: {} with error: {}",
                    ext_entry_identifier, error
                );
                return ExitCode::FAILURE;
            }
        };
    println!("Extended File System (ext) file entry information:");

    match print_ext_file_entry(&mut file_entry) {
        Ok(_) => {}
        Err(error) => {
            println!("{}", error);
            return ExitCode::FAILURE;
        }
    };
    ExitCode::SUCCESS
}

/// Prints the hierarchy of an Extended File System (ext).
pub fn print_hierarcy_ext_file_system(
    vfs_file_system: &Arc<VfsFileSystem>,
    vfs_path: &VfsPath,
    in_bodyfile_format: bool,
) -> ExitCode {
    let mut ext_file_system = ExtFileSystem::new();

    match ext_file_system.open(vfs_file_system, vfs_path) {
        Ok(_) => {}
        Err(error) => {
            println!("Unable to open ext file system with error: {}", error);
            return ExitCode::FAILURE;
        }
    };
    if in_bodyfile_format {
        println!("{}", bodyfile::BODYFILE_HEADER);
    } else {
        println!("Extended File System (ext) hierarchy:");
    }
    let mut file_entry: ExtFileEntry = match ext_file_system.get_root_directory() {
        Ok(file_entry) => file_entry,
        Err(error) => {
            println!(
                "Unable to retrieve ext root directory with error: {}",
                error
            );
            return ExitCode::FAILURE;
        }
    };
    let mut path_components: Vec<String> = Vec::new();
    match print_hierarcy_ext_file_entry(&mut file_entry, &mut path_components, in_bodyfile_format) {
        Ok(_) => {}
        Err(error) => {
            println!("{}", error);
            return ExitCode::FAILURE;
        }
    };
    ExitCode::SUCCESS
}

/// Prints the hierarchy of an Extended File System (ext) file entry.
fn print_hierarcy_ext_file_entry(
    file_entry: &mut ExtFileEntry,
    path_components: &mut Vec<String>,
    in_bodyfile_format: bool,
) -> io::Result<()> {
    if in_bodyfile_format {
        print_ext_file_entry_bodyfile(file_entry, path_components)?;
    } else {
        print_ext_file_entry_path(file_entry, path_components)?;
    }
    let number_of_file_entries: usize = file_entry.get_number_of_sub_file_entries()?;
    for sub_file_entry_index in 0..number_of_file_entries {
        let mut sub_file_entry: ExtFileEntry =
            file_entry.get_sub_file_entry_by_index(sub_file_entry_index)?;

        print_hierarcy_ext_file_entry(&mut sub_file_entry, path_components, in_bodyfile_format)?;
    }
    if file_entry.inode_number != 2 {
        path_components.pop();
    }
    Ok(())
}

/// Prints information about a specific path of an Extended File System (ext).
pub fn print_path_ext_file_system(
    vfs_file_system: &Arc<VfsFileSystem>,
    vfs_path: &VfsPath,
    path: &String,
) -> ExitCode {
    let mut ext_file_system = ExtFileSystem::new();

    match ext_file_system.open(vfs_file_system, vfs_path) {
        Ok(_) => {}
        Err(error) => {
            println!("Unable to open ext file system with error: {}", error);
            return ExitCode::FAILURE;
        }
    };
    let ext_path: ExtPath = ExtPath::from(path);
    let mut file_entry: Option<ExtFileEntry> =
        match ext_file_system.get_file_entry_by_path(&ext_path) {
            Ok(file_entry) => file_entry,
            Err(error) => {
                println!(
                    "Unable to retrive ext file entry: {} with error: {}",
                    path, error
                );
                return ExitCode::FAILURE;
            }
        };
    if file_entry.is_none() {
        println!("No such to ext file entry: {}", path);
        return ExitCode::FAILURE;
    }
    println!("Extended File System (ext) file entry information:");

    match print_ext_file_entry(file_entry.as_mut().unwrap()) {
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
        let string: String = get_file_mode_string(0x1000);
        assert_eq!(string, "p---------");

        let string: String = get_file_mode_string(0x2000);
        assert_eq!(string, "c---------");

        let string: String = get_file_mode_string(0x4000);
        assert_eq!(string, "d---------");

        let string: String = get_file_mode_string(0x6000);
        assert_eq!(string, "b---------");

        let string: String = get_file_mode_string(0xa000);
        assert_eq!(string, "l---------");

        let string: String = get_file_mode_string(0xc000);
        assert_eq!(string, "s---------");

        let string: String = get_file_mode_string(0x81ff);
        assert_eq!(string, "-rwxrwxrwx");
    }

    // TODO: add tests for print_ext_compatible_feature_flags
    // TODO: add tests for print_ext_incompatible_feature_flags
    // TODO: add tests for print_ext_read_only_compatible_feature_flags
    // TODO: add tests for print_ext_file_system
    // TODO: add tests for print_ext_file_entry
    // TODO: add tests for print_ext_file_entry_bodyfile
    // TODO: add tests for print_ext_file_entry_path
    // TODO: add tests for print_entry_ext_file_system
    // TODO: add tests for print_hierarcy_ext_file_system
    // TODO: add tests for print_hierarcy_ext_file_entry
    // TODO: add tests for print_path_ext_file_system
}
