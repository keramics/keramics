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

use std::process::ExitCode;

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_datetime::DateTime;
use keramics_formats::fat::{FatFileEntry, FatFileSystem, FatFormat, FatPath};

/// Information about a File Allocation Table (FAT).
pub struct FatInfo {}

impl FatInfo {
    /// Retrieves the string representation of a date and time value.
    fn get_date_time_string(date_time: &DateTime) -> Result<String, ErrorTrace> {
        match date_time {
            DateTime::NotSet => Ok(String::from("Not set (0)")),
            DateTime::FatDate(fat_date) => Ok(fat_date.to_iso8601_string()),
            DateTime::FatTimeDate(fat_date_time) => Ok(fat_date_time.to_iso8601_string()),
            DateTime::FatTimeDate10Ms(fat_date_time_10ms) => {
                Ok(fat_date_time_10ms.to_iso8601_string())
            }
            _ => return Err(keramics_core::error_trace_new!("Unsupported date time")),
        }
    }

    /// Retrieves string representations of file attribute flags.
    fn get_file_attribute_flags_strings(flags: u8) -> Vec<String> {
        let mut flags_strings: Vec<String> = Vec::new();
        if flags & 0x01 != 0 {
            let flag_string: String =
                String::from("0x0001: Is read-only (FILE_ATTRIBUTE_READ_ONLY)");
            flags_strings.push(flag_string);
        }
        if flags & 0x02 != 0 {
            let flag_string: String = String::from("0x0002: Is hidden (FILE_ATTRIBUTE_HIDDEN)");
            flags_strings.push(flag_string);
        }
        if flags & 0x04 != 0 {
            let flag_string: String = String::from("0x0004: Is system (FILE_ATTRIBUTE_SYSTEM)");
            flags_strings.push(flag_string);
        }

        if flags & 0x10 != 0 {
            let flag_string: String =
                String::from("0x0010: Is directory (FILE_ATTRIBUTE_DIRECTORY)");
            flags_strings.push(flag_string);
        }
        if flags & 0x20 != 0 {
            let flag_string: String =
                String::from("0x0020: Should be archived (FILE_ATTRIBUTE_ARCHIVE)");
            flags_strings.push(flag_string);
        }
        if flags & 0x40 != 0 {
            let flag_string: String = String::from("0x0040: Is device (FILE_ATTRIBUTE_DEVICE)");
            flags_strings.push(flag_string);
        }
        if flags & 0x80 != 0 {
            let flag_string: String = String::from("0x0080: Is normal (FILE_ATTRIBUTE_NORMAL)");
            flags_strings.push(flag_string);
        }
        flags_strings
    }

    /// Prints information about a file entry.
    fn print_file_entry(file_entry: &mut FatFileEntry) -> Result<(), ErrorTrace> {
        println!("    Identifier\t\t\t\t: 0x{:08x}", file_entry.identifier);

        match file_entry.get_name() {
            Some(name) => println!("    Name\t\t\t\t: {}", name.to_string()),
            None => {}
        };
        println!("    Size\t\t\t\t: {}", file_entry.get_size());

        match file_entry.get_modification_time() {
            Some(date_time) => {
                let date_time_string: String = FatInfo::get_date_time_string(date_time)?;
                println!("    Modification time\t\t\t: {}", date_time_string);
            }
            None => {}
        };
        match file_entry.get_access_time() {
            Some(date_time) => {
                let date_time_string: String = FatInfo::get_date_time_string(date_time)?;
                println!("    Access time\t\t\t\t: {}", date_time_string);
            }
            None => {}
        };
        match file_entry.get_creation_time() {
            Some(date_time) => {
                let date_time_string: String = FatInfo::get_date_time_string(date_time)?;
                println!("    Creation time\t\t\t: {}", date_time_string);
            }
            None => {}
        };
        let file_attribute_flags: u8 = file_entry.get_file_attribute_flags();
        println!(
            "    File attribute flags\t\t: 0x{:02x}",
            file_attribute_flags
        );
        let flags_strings: Vec<String> =
            Self::get_file_attribute_flags_strings(file_attribute_flags);
        println!(
            "{}",
            flags_strings
                .iter()
                .map(|string| format!("        {}", string))
                .collect::<Vec<String>>()
                .join("\n")
        );
        if file_attribute_flags != 0 {
            println!("");
        }
        Ok(())
    }

    /// Prints information about a specific file entry.
    pub fn print_file_entry_by_identifier(
        data_stream: &DataStreamReference,
        fat_entry_identifier: u64,
    ) -> ExitCode {
        let mut fat_file_system = FatFileSystem::new();

        match fat_file_system.read_data_stream(data_stream) {
            Ok(_) => {}
            Err(error) => {
                println!("Unable to open file system with error: {}", error);
                return ExitCode::FAILURE;
            }
        }
        if fat_entry_identifier > u32::MAX as u64 {
            println!(
                "Invalid identifier: 0x{:08x} value out of bounds",
                fat_entry_identifier
            );
            return ExitCode::FAILURE;
        }
        let mut file_entry: FatFileEntry =
            match fat_file_system.get_file_entry_by_identifier(fat_entry_identifier as u32) {
                Ok(file_entry) => file_entry,
                Err(error) => {
                    println!(
                        "Unable to retrive file entry: 0x{:08x} with error: {}",
                        fat_entry_identifier, error
                    );
                    return ExitCode::FAILURE;
                }
            };
        println!("File Allocation Table (FAT) file entry information:");

        match Self::print_file_entry(&mut file_entry) {
            Ok(_) => {}
            Err(error) => {
                println!("{}", error);
                return ExitCode::FAILURE;
            }
        };
        ExitCode::SUCCESS
    }

    /// Prints information about a specific file entry.
    pub fn print_file_entry_by_path(
        data_stream: &DataStreamReference,
        path_components: &[&str],
    ) -> ExitCode {
        let mut fat_file_system = FatFileSystem::new();

        match fat_file_system.read_data_stream(data_stream) {
            Ok(_) => {}
            Err(error) => {
                println!("Unable to open file system with error: {}", error);
                return ExitCode::FAILURE;
            }
        }
        let fat_path: FatPath = FatPath::from(path_components);

        let mut file_entry: Option<FatFileEntry> =
            match fat_file_system.get_file_entry_by_path(&fat_path) {
                Ok(file_entry) => file_entry,
                Err(error) => {
                    println!("Unable to retrive file entry with error: {}", error);
                    return ExitCode::FAILURE;
                }
            };
        if file_entry.is_none() {
            println!("No such file entry");
            return ExitCode::FAILURE;
        }
        println!("File Allocation Table (FAT) file entry information:");

        match Self::print_file_entry(file_entry.as_mut().unwrap()) {
            Ok(_) => {}
            Err(error) => {
                println!("{}", error);
                return ExitCode::FAILURE;
            }
        };
        ExitCode::SUCCESS
    }

    /// Prints the file system hierarchy.
    pub fn print_hierarchy(data_stream: &DataStreamReference) -> ExitCode {
        let mut fat_file_system = FatFileSystem::new();

        match fat_file_system.read_data_stream(data_stream) {
            Ok(_) => {}
            Err(error) => {
                println!("Unable to open file system with error: {}", error);
                return ExitCode::FAILURE;
            }
        }
        println!("File Allocation Table (FAT) hierarchy:");

        let mut file_entry: FatFileEntry = match fat_file_system.get_root_directory() {
            Ok(file_entry) => file_entry,
            Err(error) => {
                println!("{}", error);
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
        };
        ExitCode::SUCCESS
    }

    /// Prints the file entry hierarchy.
    fn print_hierarchy_file_entry(
        file_entry: &mut FatFileEntry,
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
        println!("{}", path);

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
            let mut sub_file_entry: FatFileEntry =
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

    /// Prints information about the file system.
    pub fn print_file_system(data_stream: &DataStreamReference) -> ExitCode {
        let mut fat_file_system = FatFileSystem::new();

        match fat_file_system.read_data_stream(data_stream) {
            Ok(_) => {}
            Err(error) => {
                println!("Unable to open file system with error: {}", error);
                return ExitCode::FAILURE;
            }
        }
        println!("File Allocation Table (FAT) information:");

        let format_version: u8 = match &fat_file_system.format {
            FatFormat::Fat12 => 12,
            FatFormat::Fat16 => 16,
            FatFormat::Fat32 => 32,
        };
        println!("    Format version\t\t\t: FAT-{}", format_version);

        let volume_label: String = match fat_file_system.get_volume_label() {
            Some(volume_label) => volume_label.to_string(),
            None => String::new(),
        };
        println!("    Volume label\t\t\t: {}", volume_label);

        println!("");

        ExitCode::SUCCESS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: add tests for get_date_time_string
    // TODO: add tests for print_file_entry
    // TODO: add tests for print_file_entry_by_identifier
    // TODO: add tests for print_file_entry_by_path
    // TODO: add tests for print_hierarchy
    // TODO: add tests for print_hierarchy_file_entry
    // TODO: add tests for print_volume_system
}
