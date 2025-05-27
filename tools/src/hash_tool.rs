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
use std::io::{BufReader, Stdin};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, ValueEnum};

use core::formatters::format_as_string;
use core::mediator::Mediator;
use core::{open_os_data_stream, DataStreamReference};
use types::Ucs2String;
use vfs::{
    VfsDataFork, VfsFileEntry, VfsFileSystemReference, VfsFinder, VfsPath, VfsResolver,
    VfsResolverReference, VfsScanContext, VfsScanNode, VfsScanner, VfsString,
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
}

/// Tool for calculating digest hashes of data streams.
struct HashTool {
    /// The digest hasher.
    pub digest_hasher: hasher::DigestHasher,

    /// Character translation table.
    translation_table: HashMap<u32, String>,
}

impl HashTool {
    /// Creates a new tool.
    fn new(hash_type: &HashType) -> Self {
        let digest_hash_type: hasher::DigestHashType = match hash_type {
            HashType::Md5 => hasher::DigestHashType::Md5,
            HashType::Sha1 => hasher::DigestHashType::Sha1,
            HashType::Sha224 => hasher::DigestHashType::Sha224,
            HashType::Sha256 => hasher::DigestHashType::Sha256,
            HashType::Sha512 => hasher::DigestHashType::Sha512,
        };
        Self {
            digest_hasher: hasher::DigestHasher::new(&digest_hash_type),
            translation_table: HashTool::get_character_translation_table(),
        }
    }

    /// Calculates a digest hash from a file entry.
    fn calculate_hash_from_file_entry(
        &self,
        file_entry: &VfsFileEntry,
        file_system_display_path: &String,
        path_components: &Vec<VfsString>,
    ) -> io::Result<()> {
        let number_of_data_forks: usize = file_entry.get_number_of_data_forks()?;

        for data_fork_index in 0..number_of_data_forks {
            let data_fork: VfsDataFork = file_entry.get_data_fork_by_index(data_fork_index)?;

            let name: Option<String> = match data_fork.get_name() {
                Some(name) => Some(name.to_string()),
                None => None,
            };
            // TODO: create skip list
            let hash_string: String = if path_components.len() > 1
                && path_components[1] == VfsString::Ucs2(Ucs2String::from_string("$BadClus"))
                && name == Some("$Bad".to_string())
            {
                String::from("N/A")
            } else {
                let data_stream: DataStreamReference = data_fork.get_data_stream()?;
                let hash: Vec<u8> = self
                    .digest_hasher
                    .calculate_hash_from_data_stream(&data_stream)?;

                format_as_string(&hash)
            };
            let path: String = path_components
                .iter()
                .map(|component| component.to_string())
                .collect::<Vec<String>>()
                .join("/");
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
    fn calculate_hash_from_scan_node(&self, vfs_scan_node: &VfsScanNode) -> io::Result<()> {
        if vfs_scan_node.is_empty() {
            // Only process scan nodes that contain a file system.
            match &vfs_scan_node.path {
                VfsPath::Ext { .. } | VfsPath::Ntfs { .. } => {}
                _ => return Ok(()),
            }
            let vfs_resolver: VfsResolverReference = VfsResolver::current();

            let file_system: VfsFileSystemReference =
                vfs_resolver.open_file_system(&vfs_scan_node.path)?;

            for result in VfsFinder::new(&file_system) {
                match result {
                    Ok((file_entry, path_components)) => {
                        let display_path: String = match vfs_scan_node.path.get_parent() {
                            Some(parent_path) => self.get_display_path(parent_path),
                            None => String::new(),
                        };
                        self.calculate_hash_from_file_entry(
                            &file_entry,
                            &display_path,
                            &path_components,
                        )?
                    }
                    Err(error) => return Err(error),
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

    /// Retrieves a string representation of a path.
    fn get_display_path(&self, path: &VfsPath) -> String {
        // TODO: add support for aliases
        // TODO: santize path (control characters, etc.)
        match path {
            VfsPath::Apm { location, .. } => location.replace("apm", "p"),
            VfsPath::Ext { ext_path, parent } => {
                let parent_display_path: String = self.get_display_path(parent);
                let location: String = ext_path.to_string();
                format!("{}{}", parent_display_path, location)
            }
            VfsPath::Gpt { location, .. } => location.replace("gpt", "p"),
            VfsPath::Mbr { location, .. } => location.replace("mbr", "p"),
            VfsPath::Ntfs { ntfs_path, parent } => {
                let parent_display_path: String = self.get_display_path(parent);
                let location: String = ntfs_path.to_string();
                format!("{}{}", parent_display_path, location)
            }
            _ => String::new(),
        }
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
}

fn main() -> ExitCode {
    let arguments = CommandLineArguments::parse();

    Mediator {
        debug_output: arguments.debug,
    }
    .make_current();

    let hash_tool: HashTool = HashTool::new(&arguments.digest_hash_type);

    match arguments.source {
        None => {
            let mut reader: BufReader<Stdin> = BufReader::new(io::stdin());

            let hash: Vec<u8> = hash_tool
                .digest_hasher
                .calculate_hash_from_reader(&mut reader);
            println!("{}  -", format_as_string(&hash));
        }
        Some(source_argument) => {
            let source: &str = match source_argument.to_str() {
                Some(value) => value,
                None => {
                    println!("Missing source");
                    return ExitCode::FAILURE;
                }
            };
            let vfs_path: VfsPath = VfsPath::Os {
                location: source.to_string(),
            };

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
            match vfs_scanner.scan(&mut vfs_scan_context, &vfs_path) {
                Ok(_) => {}
                Err(error) => {
                    println!("Unable to scan: {} with error: {}", source, error);
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
                let data_stream: DataStreamReference = match open_os_data_stream(source) {
                    Ok(data_stream) => data_stream,
                    Err(error) => {
                        println!("Unable to open file: {} with error: {}", source, error);
                        return ExitCode::FAILURE;
                    }
                };
                let hash: Vec<u8> = match hash_tool
                    .digest_hasher
                    .calculate_hash_from_data_stream(&data_stream)
                {
                    Ok(hash) => hash,
                    Err(error) => {
                        println!(
                            "Unable to calculate hash from data stream with error: {}",
                            error
                        );
                        return ExitCode::FAILURE;
                    }
                };
                println!("{}  {}", format_as_string(&hash), source);
            } else {
                match hash_tool.calculate_hash_from_scan_node(root_scan_node) {
                    Ok(_) => {}
                    Err(error) => {
                        println!(
                            "Unable to calculate hash from root scan node with error: {}",
                            error
                        );
                        return ExitCode::FAILURE;
                    }
                };
            }
        }
    };
    ExitCode::SUCCESS
}
