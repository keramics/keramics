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
use std::io::{BufReader, Stdin};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, ValueEnum};

use keramics_core::formatters::format_as_string;
use keramics_core::mediator::Mediator;
use keramics_core::{DataStreamReference, ErrorTrace, open_os_data_stream};
use keramics_hashes::{
    DigestHashContext, Md5Context, Sha1Context, Sha224Context, Sha256Context, Sha512Context,
};
use keramics_types::Ucs2String;
use keramics_vfs::{
    VfsDataFork, VfsFileEntry, VfsFileSystemReference, VfsFinder, VfsLocation, VfsResolver,
    VfsResolverReference, VfsScanContext, VfsScanNode, VfsScanner, VfsString, VfsType,
    new_os_vfs_location,
};

mod hasher;

#[derive(Clone, ValueEnum)]
enum HashType {
    /// MD5
    Md5,

    /// SHA1
    Sha1,

    /// SHA-224
    Sha224,

    /// SHA-256
    Sha256,

    /// SHA-512
    Sha512,
}

#[derive(Clone, ValueEnum)]
enum VolumePathType {
    /// Identifier based volume or partition path, such as /apfs{f449e580-e355-4e74-8880-05e46e4e3b1e}
    Identifier,

    /// Index based volume or partition path, such as /apfs1 or /p1
    Index,
}

#[derive(Parser)]
#[command(version, about = "Calculate digest hashes of data streams", long_about = None)]
struct CommandLineArguments {
    #[arg(long, default_value_t = false)]
    /// Enable debug output
    debug: bool,

    /// Digest hash type
    #[arg(short, long, default_value_t = HashType::Md5, value_enum)]
    digest_hash_type: HashType,

    /// Path of the file to read the data from, if not provided the data will be read from standard input
    source: Option<PathBuf>,

    #[arg(long, default_value_t = false)]
    /// Stop when an error is encountered
    stop_on_error: bool,

    /// Volume or partition path type
    #[arg(long, default_value_t = VolumePathType::Index, value_enum)]
    volume_path_type: VolumePathType,
}

// TODO: move DigestHasher into HashTool

/// Tool for calculating digest hashes of data streams.
struct HashTool {
    /// The digest hasher.
    pub digest_hasher: hasher::DigestHasher,

    /// The digest hash type.
    hash_type: HashType,

    /// Character translation table.
    translation_table: HashMap<u32, String>,

    /// Volume or partition path type
    volume_path_type: VolumePathType,

    /// Value to indicate to stop on error.
    pub stop_on_error: bool,
}

impl HashTool {
    const READ_BUFFER_SIZE: usize = 65536;

    /// Creates a new tool.
    fn new(hash_type: &HashType, volume_path_type: &VolumePathType, stop_on_error: bool) -> Self {
        let digest_hash_type: hasher::DigestHashType = match hash_type {
            HashType::Md5 => hasher::DigestHashType::Md5,
            HashType::Sha1 => hasher::DigestHashType::Sha1,
            HashType::Sha224 => hasher::DigestHashType::Sha224,
            HashType::Sha256 => hasher::DigestHashType::Sha256,
            HashType::Sha512 => hasher::DigestHashType::Sha512,
        };
        Self {
            digest_hasher: hasher::DigestHasher::new(&digest_hash_type),
            hash_type: hash_type.clone(),
            translation_table: HashTool::get_character_translation_table(),
            volume_path_type: volume_path_type.clone(),
            stop_on_error: stop_on_error,
        }
    }

    /// Calculates a digest hash from a data fork.
    fn calculate_hash_from_data_fork(
        &self,
        data_fork: &VfsDataFork,
        path_components: &Vec<VfsString>,
    ) -> Result<String, ErrorTrace> {
        let data_stream: DataStreamReference = match data_fork.get_data_stream() {
            Ok(data_stream) => data_stream,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to retrieve data stream from data fork"
                );
                return Err(error);
            }
        };
        let hash_string: String = match self.calculate_hash_from_data_stream(&data_stream) {
            Ok(hash) => hash,
            Err(mut error) => {
                if self.stop_on_error {
                    let path: String = self.get_path(path_components);

                    let escaped_path_and_name: String = match data_fork.get_name() {
                        Some(fork_name) => {
                            let name: String = fork_name.to_string();
                            let escaped_name: String = self.get_escaped_path(&name);
                            let escaped_path: String = self.get_escaped_path(&path);

                            [escaped_path, escaped_name].join(":")
                        }
                        None => self.get_escaped_path(&path),
                    };
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to calculate hash of data stream: {}",
                            escaped_path_and_name
                        )
                    );
                    return Err(error);
                }
                String::from("N/A (error)")
            }
        };
        Ok(hash_string)
    }

    /// Calculates a digest hash from a data stream.
    fn calculate_hash_from_data_stream(
        &self,
        data_stream: &DataStreamReference,
    ) -> Result<String, ErrorTrace> {
        let mut hash_context: Box<dyn DigestHashContext> = match &self.hash_type {
            HashType::Md5 => Box::new(Md5Context::new()),
            HashType::Sha1 => Box::new(Sha1Context::new()),
            HashType::Sha224 => Box::new(Sha224Context::new()),
            HashType::Sha256 => Box::new(Sha256Context::new()),
            HashType::Sha512 => Box::new(Sha512Context::new()),
        };
        let mut data: [u8; HashTool::READ_BUFFER_SIZE] = [0; HashTool::READ_BUFFER_SIZE];

        match data_stream.write() {
            Ok(mut data_stream) => loop {
                let read_count = match data_stream.read(&mut data) {
                    Ok(read_count) => read_count,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to read data stream");
                        return Err(error);
                    }
                };
                if read_count == 0 {
                    break;
                }
                hash_context.update(&data[0..read_count]);
            },
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to obtain write lock on data stream",
                    error
                ));
            }
        };
        let hash: Vec<u8> = hash_context.finalize();

        Ok(format_as_string(&hash))
    }

    /// Calculates a digest hash from a file entry.
    fn calculate_hash_from_file_entry(
        &self,
        file_entry: &VfsFileEntry,
        file_system_display_path: &String,
        path_components: &Vec<VfsString>,
    ) -> Result<(), ErrorTrace> {
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
            let data_fork: VfsDataFork = match file_entry.get_data_fork_by_index(data_fork_index) {
                Ok(data_fork) => data_fork,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to retrieve data fork: {}", data_fork_index)
                    );
                    return Err(error);
                }
            };
            let name: Option<String> = match data_fork.get_name() {
                Some(fork_name) => Some(fork_name.to_string()),
                None => None,
            };
            // TODO: create skip list
            let hash_string: String = if path_components.len() > 1
                && path_components[1] == VfsString::Ucs2(Ucs2String::from("$BadClus"))
                && name == Some(String::from("$Bad"))
            {
                String::from("N/A (skipped)")
            } else {
                self.calculate_hash_from_data_fork(&data_fork, path_components)?
            };
            let path: String = self.get_path(path_components);
            let escaped_path: String = self.get_escaped_path(&path);

            match name {
                Some(name) => {
                    let escaped_name: String = self.get_escaped_path(&name);
                    println!(
                        "{}\t{}{}:{}",
                        hash_string, file_system_display_path, escaped_path, escaped_name
                    );
                }
                None => println!(
                    "{}\t{}{}",
                    hash_string, file_system_display_path, escaped_path
                ),
            };
        }
        Ok(())
    }

    /// Calculates a digest hash from a scan node.
    fn calculate_hash_from_scan_node(&self, vfs_scan_node: &VfsScanNode) -> Result<(), ErrorTrace> {
        if vfs_scan_node.is_empty() {
            // Only process scan nodes that contain a file system.
            match vfs_scan_node.get_type() {
                VfsType::Ext { .. } | VfsType::Ntfs { .. } => {}
                _ => return Ok(()),
            }
            let vfs_resolver: VfsResolverReference = VfsResolver::current();

            let file_system: VfsFileSystemReference =
                match vfs_resolver.open_file_system(&vfs_scan_node.location) {
                    Ok(file_system) => file_system,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to open file system");
                        return Err(error);
                    }
                };

            let display_path: String = match vfs_scan_node.location.get_parent() {
                Some(parent_path) => self.get_display_path(parent_path)?,
                None => String::new(),
            };
            for result in VfsFinder::new(&file_system) {
                match result {
                    Ok((file_entry, path_components)) => self.calculate_hash_from_file_entry(
                        &file_entry,
                        &display_path,
                        &path_components,
                    )?,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to retrieve file entry from finder"
                        );
                        return Err(error);
                    }
                };
            }
        } else {
            for sub_scan_node in vfs_scan_node.sub_nodes.iter() {
                self.calculate_hash_from_scan_node(sub_scan_node)?;
            }
        }
        Ok(())
    }

    /// Retrieves a character translation table.
    fn get_character_translation_table() -> HashMap<u32, String> {
        let mut translation_table: HashMap<u32, String> = HashMap::new();

        // Escape C0 control characters as \x##
        for character_value in 0x00..0x20 {
            let escaped_character: String = format!("\\x{:02x}", character_value);
            translation_table.insert(character_value, escaped_character);
        }
        // Escape C1 control character as \x##
        for character_value in 0x7f..0xa0 {
            let escaped_character: String = format!("\\x{:02x}", character_value);
            translation_table.insert(character_value, escaped_character);
        }
        // Escape Unicode surrogate characters as \U########
        for character_value in 0xd800..0xe000 {
            let escaped_character: String = format!("\\U{:08x}", character_value);
            translation_table.insert(character_value, escaped_character);
        }
        // Escape undefined Unicode characters as \U########
        let character_values: Vec<u32> = vec![
            0xfdd0, 0xfdd1, 0xfdd2, 0xfdd3, 0xfdd4, 0xfdd5, 0xfdd6, 0xfdd7, 0xfdd8, 0xfdd9, 0xfdda,
            0xfddb, 0xfddc, 0xfddd, 0xfdde, 0xfddf, 0xfffe, 0xffff, 0x1fffe, 0x1ffff, 0x2fffe,
            0x2ffff, 0x3fffe, 0x3ffff, 0x4fffe, 0x4ffff, 0x5fffe, 0x5ffff, 0x6fffe, 0x6ffff,
            0x7fffe, 0x7ffff, 0x8fffe, 0x8ffff, 0x9fffe, 0x9ffff, 0xafffe, 0xaffff, 0xbfffe,
            0xbffff, 0xcfffe, 0xcffff, 0xdfffe, 0xdffff, 0xefffe, 0xeffff, 0xffffe, 0xfffff,
            0x10fffe, 0x10ffff,
        ];
        for character_value in character_values.iter() {
            let escaped_character: String = format!("\\U{:08x}", character_value);
            translation_table.insert(*character_value, escaped_character);
        }
        // Escape observed non-printable Unicode characters as \U########
        let character_values: Vec<u32> = vec![
            0x2028, 0x2029, 0xe000, 0xf8ff, 0xf0000, 0xffffd, 0x100000, 0x10fffd,
        ];
        for character_value in character_values.iter() {
            let escaped_character: String = format!("\\U{:08x}", character_value);
            translation_table.insert(*character_value, escaped_character);
        }
        translation_table
    }

    /// Retrieves a human readable path representation of a VFS location.
    fn get_display_path(&self, vfs_location: &VfsLocation) -> Result<String, ErrorTrace> {
        match &self.volume_path_type {
            VolumePathType::Identifier => self.get_identifier_display_path(vfs_location),
            VolumePathType::Index => self.get_index_display_path(vfs_location),
        }
        // TODO: santize path (control characters, etc.)
    }

    /// Retrieves an escaped path.
    fn get_escaped_path(&self, path: &String) -> String {
        let mut string_parts: Vec<String> = Vec::new();
        for character_value in path.chars() {
            let safe_character: String = match self.translation_table.get(&(character_value as u32))
            {
                Some(escaped_character) => escaped_character.clone(),
                None => character_value.to_string(),
            };
            string_parts.push(safe_character);
        }
        string_parts.join("")
    }

    /// Retrieves an identifier-based a human readable path representation of a VFS location.
    fn get_identifier_display_path(
        &self,
        vfs_location: &VfsLocation,
    ) -> Result<String, ErrorTrace> {
        let vfs_resolver: VfsResolverReference = VfsResolver::current();
        let result: Option<VfsFileEntry> = match vfs_resolver.get_file_entry_by_path(vfs_location) {
            Ok(file_entry) => file_entry,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to retrieve file entry");
                return Err(error);
            }
        };
        let display_path: Option<String> = match result {
            Some(VfsFileEntry::Gpt(gpt_file_entry)) => match gpt_file_entry.get_identifier() {
                Some(identifier) => Some(format!("/gpt{{{}}}", identifier.to_string())),
                _ => None,
            },
            _ => None,
        };
        match display_path {
            Some(display_path) => Ok(display_path),
            None => self.get_index_display_path(vfs_location),
        }
    }

    /// Retrieves an index-based a string representation of a VFS location.
    fn get_index_display_path(&self, vfs_location: &VfsLocation) -> Result<String, ErrorTrace> {
        let display_path: String = match vfs_location {
            VfsLocation::Layer {
                path,
                parent,
                vfs_type,
            } => {
                let path_string: String = path.to_string();
                match vfs_type {
                    VfsType::Apm => path_string.replace("apm", "p"),
                    VfsType::Ext => {
                        let parent_display_path: String = self.get_display_path(parent)?;
                        format!("{}{}", parent_display_path, path_string)
                    }
                    VfsType::Gpt => path_string.replace("gpt", "p"),
                    VfsType::Ntfs => {
                        let parent_display_path: String = self.get_display_path(parent)?;
                        format!("{}{}", parent_display_path, path_string)
                    }
                    VfsType::Mbr => path_string.replace("mbr", "p"),
                    _ => String::new(),
                }
            }
            _ => String::new(),
        };
        Ok(display_path)
    }

    /// Retrieves a path string based on the path components.
    fn get_path(&self, path_components: &Vec<VfsString>) -> String {
        path_components
            .iter()
            .map(|component| component.to_string())
            .collect::<Vec<String>>()
            .join("/")
    }
}

fn main() -> ExitCode {
    let arguments = CommandLineArguments::parse();

    Mediator {
        debug_output: arguments.debug,
    }
    .make_current();

    let hash_tool: HashTool = HashTool::new(
        &arguments.digest_hash_type,
        &arguments.volume_path_type,
        arguments.stop_on_error,
    );
    match arguments.source {
        None => {
            let mut reader: BufReader<Stdin> = BufReader::new(std::io::stdin());

            let hash_string: String = match hash_tool
                .digest_hasher
                .calculate_hash_from_reader(&mut reader)
            {
                Ok(hash) => hash,
                Err(error) => {
                    println!("Unable to calculate hash from stdin with error:\n{}", error);
                    return ExitCode::FAILURE;
                }
            };
            println!("{}  -", hash_string);
        }
        Some(source_argument) => {
            let source: &str = match source_argument.to_str() {
                Some(value) => value,
                None => {
                    println!("Missing source");
                    return ExitCode::FAILURE;
                }
            };
            let vfs_location: VfsLocation = new_os_vfs_location(source);

            // TODO: add scanner options.
            // TODO: add scanner mediator.
            let mut vfs_scanner: VfsScanner = VfsScanner::new();
            match vfs_scanner.build() {
                Ok(_) => {}
                Err(error) => {
                    println!("{}", error);
                    return ExitCode::FAILURE;
                }
            };
            let mut vfs_scan_context: VfsScanContext = VfsScanContext::new();
            match vfs_scanner.scan(&mut vfs_scan_context, &vfs_location) {
                Ok(_) => {}
                Err(error) => {
                    println!("Unable to scan: {} with error:\n{}", source, error);
                    return ExitCode::FAILURE;
                }
            };
            let root_scan_node: &VfsScanNode = match vfs_scan_context.root_node.as_ref() {
                Some(scan_node) => scan_node,
                None => {
                    println!("Unable to scan: {} missing root scan node", source);
                    return ExitCode::FAILURE;
                }
            };
            if root_scan_node.is_empty() {
                let data_stream: DataStreamReference = match open_os_data_stream(&source_argument) {
                    Ok(data_stream) => data_stream,
                    Err(error) => {
                        println!("Unable to open file: {} with error:\n{}", source, error);
                        return ExitCode::FAILURE;
                    }
                };
                let hash_string: String =
                    match hash_tool.calculate_hash_from_data_stream(&data_stream) {
                        Ok(hash) => hash,
                        Err(error) => {
                            if hash_tool.stop_on_error {
                                println!(
                                    "Unable to calculate hash of: {} with error:\n{}",
                                    source, error
                                );
                                return ExitCode::FAILURE;
                            }
                            String::from("N/A (error)")
                        }
                    };
                println!("{}  {}", hash_string, source);
            } else {
                match hash_tool.calculate_hash_from_scan_node(root_scan_node) {
                    Ok(_) => {}
                    Err(error) => {
                        println!(
                            "Unable to calculate hash of: {} with error:\n{}",
                            source, error
                        );
                        return ExitCode::FAILURE;
                    }
                };
            }
        }
    };
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_core::{DataStreamReference, open_fake_data_stream};

    #[test]
    fn test_calculate_md5() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = vec![
            0x41, 0x20, 0x63, 0x65, 0x72, 0x61, 0x6d, 0x69, 0x63, 0x20, 0x69, 0x73, 0x20, 0x61,
            0x6e, 0x79, 0x20, 0x6f, 0x66, 0x20, 0x74, 0x68, 0x65, 0x20, 0x76, 0x61, 0x72, 0x69,
            0x6f, 0x75, 0x73, 0x20, 0x68, 0x61, 0x72, 0x64, 0x2c, 0x20, 0x62, 0x72, 0x69, 0x74,
            0x74, 0x6c, 0x65, 0x2c, 0x20, 0x68, 0x65, 0x61, 0x74, 0x2d, 0x72, 0x65, 0x73, 0x69,
            0x73, 0x74, 0x61, 0x6e, 0x74, 0x2c, 0x20, 0x61, 0x6e, 0x64, 0x20, 0x63, 0x6f, 0x72,
            0x72, 0x6f, 0x73, 0x69, 0x6f, 0x6e, 0x2d, 0x72, 0x65, 0x73, 0x69, 0x73, 0x74, 0x61,
            0x6e, 0x74, 0x20, 0x6d, 0x61, 0x74, 0x65, 0x72, 0x69, 0x61, 0x6c, 0x73, 0x20, 0x6d,
            0x61, 0x64, 0x65, 0x20, 0x62, 0x79, 0x20, 0x73, 0x68, 0x61, 0x70, 0x69, 0x6e, 0x67,
            0x20, 0x61, 0x6e, 0x64, 0x20, 0x74, 0x68, 0x65, 0x6e, 0x20, 0x66, 0x69, 0x72, 0x69,
            0x6e, 0x67, 0x20, 0x61, 0x6e, 0x20, 0x69, 0x6e, 0x6f, 0x72, 0x67, 0x61, 0x6e, 0x69,
            0x63, 0x2c, 0x20, 0x6e, 0x6f, 0x6e, 0x6d, 0x65, 0x74, 0x61, 0x6c, 0x6c, 0x69, 0x63,
            0x20, 0x6d, 0x61, 0x74, 0x65, 0x72, 0x69, 0x61, 0x6c, 0x2c, 0x20, 0x73, 0x75, 0x63,
            0x68, 0x20, 0x61, 0x73, 0x20, 0x63, 0x6c, 0x61, 0x79, 0x2c, 0x20, 0x61, 0x74, 0x20,
            0x61, 0x20, 0x68, 0x69, 0x67, 0x68, 0x20, 0x74, 0x65, 0x6d, 0x70, 0x65, 0x72, 0x61,
            0x74, 0x75, 0x72, 0x65, 0x2e, 0x0a,
        ];
        let data_stream: DataStreamReference = open_fake_data_stream(test_data);

        let hash_tool: HashTool = HashTool::new(&HashType::Md5, &VolumePathType::Index, true);
        let md5: String = hash_tool.calculate_hash_from_data_stream(&data_stream)?;
        assert_eq!(md5, "f19106bcf25fa9cabc1b5ac91c726001");

        Ok(())
    }

    // TODO: add more tests
}
