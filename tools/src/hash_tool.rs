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
use std::io::{BufReader, Stdin};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, ValueEnum};

use core::formatters::format_as_string;
use core::mediator::Mediator;
use core::DataStreamReference;
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
#[command(version, about = "Calculate a digest hash of data", long_about = None)]
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

/// Returns a string representation of a path.
fn get_display_path(path: &VfsPath) -> String {
    // TODO: add support for aliases
    match path {
        VfsPath::Apm { location, .. } => location.replace("apm", "p"),
        VfsPath::Ext { ext_path, parent } => {
            let parent_display_path: String = get_display_path(parent);
            let location: String = ext_path.to_string();
            format!("{}{}", parent_display_path, location)
        }
        VfsPath::Gpt { location, .. } => location.clone(),
        VfsPath::Mbr { location, .. } => location.replace("mbr", "p"),
        VfsPath::Ntfs { ntfs_path, parent } => {
            let parent_display_path: String = get_display_path(parent);
            let location: String = ntfs_path.to_string();
            format!("{}{}", parent_display_path, location)
        }
        _ => String::new(),
    }
}

/// Calculates a digest hash from a file entry.
fn calculate_hash_from_file_entry(
    digest_hasher: &hasher::DigestHasher,
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
            let hash: Vec<u8> = digest_hasher.calculate_hash_from_data_stream(&data_stream)?;

            format_as_string(&hash)
        };
        let path: String = path_components
            .iter()
            .map(|component| component.to_string())
            .collect::<Vec<String>>()
            .join("/");
        match name {
            Some(name) => println!(
                "{}\t{}{}:{}",
                hash_string, file_system_display_path, path, name
            ),
            None => println!("{}\t{}{}", hash_string, file_system_display_path, path),
        };
    }
    Ok(())
}

/// Calculates a digest hash from a scan node.
fn calculate_hash_from_scan_node(
    digest_hasher: &hasher::DigestHasher,
    vfs_scan_node: &VfsScanNode,
) -> io::Result<()> {
    if vfs_scan_node.sub_nodes.is_empty() {
        let vfs_resolver: VfsResolverReference = VfsResolver::current();

        let file_system: VfsFileSystemReference =
            vfs_resolver.open_file_system(&vfs_scan_node.path)?;

        for result in VfsFinder::new(&file_system) {
            match result {
                Ok((file_entry, path_components)) => {
                    let display_path: String = match vfs_scan_node.path.get_parent() {
                        Some(parent_path) => get_display_path(parent_path),
                        None => String::new(),
                    };
                    calculate_hash_from_file_entry(
                        digest_hasher,
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
            calculate_hash_from_scan_node(digest_hasher, sub_scan_node)?;
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    let arguments = CommandLineArguments::parse();

    Mediator {
        debug_output: arguments.debug,
    }
    .make_current();

    let digest_hash_type: hasher::DigestHashType = match &arguments.digest_hash_type {
        HashType::Md5 => hasher::DigestHashType::Md5,
        HashType::Sha1 => hasher::DigestHashType::Sha1,
        HashType::Sha224 => hasher::DigestHashType::Sha224,
        HashType::Sha256 => hasher::DigestHashType::Sha256,
        HashType::Sha512 => hasher::DigestHashType::Sha512,
    };
    let digest_hasher: hasher::DigestHasher = hasher::DigestHasher::new(&digest_hash_type);

    match arguments.source {
        None => {
            let mut reader: BufReader<Stdin> = BufReader::new(io::stdin());

            let hash: Vec<u8> = digest_hasher.calculate_hash_from_reader(&mut reader);
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
            if root_scan_node.sub_nodes.is_empty() {
                let vfs_resolver: VfsResolverReference = VfsResolver::current();

                // TODO: add support for non-default data stream.
                let result: Option<DataStreamReference> = match vfs_resolver
                    .get_data_stream_by_path_and_name(&root_scan_node.path, None)
                {
                    Ok(result) => result,
                    Err(error) => {
                        println!("Unable to open data stream with error: {}", error);
                        return ExitCode::FAILURE;
                    }
                };
                let data_stream: DataStreamReference = match result {
                    Some(data_stream) => data_stream,
                    None => {
                        println!("No such file: {}", source);
                        return ExitCode::FAILURE;
                    }
                };
                let hash: Vec<u8> =
                    match digest_hasher.calculate_hash_from_data_stream(&data_stream) {
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
                match calculate_hash_from_scan_node(&digest_hasher, root_scan_node) {
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
