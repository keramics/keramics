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

use std::io::{BufReader, Stdin};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;

use keramics_core::formatters::format_as_string;
use keramics_core::mediator::Mediator;
use keramics_core::{DataStreamReference, ErrorTrace, open_os_data_stream};
use keramics_hashes::{
    DigestHashContext, Md5Context, Sha1Context, Sha224Context, Sha256Context, Sha512Context,
};
use keramics_types::Ucs2String;
use keramics_vfs::{
    VfsDataFork, VfsFileEntry, VfsFileSystemReference, VfsFinder, VfsLocation, VfsResolver,
    VfsResolverReference, VfsScanContext, VfsScanNode, VfsScanner, VfsString, new_os_vfs_location,
};

mod display_path;
mod enums;
mod hasher;

use crate::display_path::DisplayPath;
use crate::enums::{DigestHashType, DisplayPathType};
use crate::hasher::DigestHasher;

#[derive(Parser)]
#[command(version, about = "Calculate digest hashes of data streams", long_about = None)]
struct CommandLineArguments {
    #[arg(long, default_value_t = false)]
    /// Enable debug output
    debug: bool,

    /// Digest hash type
    #[arg(short, long, default_value_t = DigestHashType::Md5, value_enum)]
    digest_hash_type: DigestHashType,

    /// Path of the file to read the data from, if not provided the data will be read from standard input
    source: Option<PathBuf>,

    #[arg(long, default_value_t = false)]
    /// Stop when an error is encountered
    stop_on_error: bool,

    /// Volume or partition path type
    #[arg(long, default_value_t = DisplayPathType::Index, value_enum)]
    volume_path_type: DisplayPathType,
}

// TODO: move DigestHasher into HashTool

/// Tool for calculating digest hashes of data streams.
struct HashTool {
    /// The digest hasher.
    pub digest_hasher: DigestHasher,

    /// The display path.
    display_path: DisplayPath,

    /// The digest hash type.
    digest_hash_type: DigestHashType,

    /// Value to indicate to stop on error.
    pub stop_on_error: bool,
}

impl HashTool {
    const READ_BUFFER_SIZE: usize = 65536;

    /// Creates a new tool.
    fn new(
        digest_hash_type: &DigestHashType,
        display_path_type: &DisplayPathType,
        stop_on_error: bool,
    ) -> Self {
        Self {
            digest_hasher: DigestHasher::new(digest_hash_type),
            display_path: DisplayPath::new(display_path_type),
            digest_hash_type: digest_hash_type.clone(),
            stop_on_error: stop_on_error,
        }
    }

    /// Calculates a digest hash from a data fork.
    fn calculate_hash_from_data_fork(&self, data_fork: &VfsDataFork) -> Result<String, ErrorTrace> {
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
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to calculate hash of data stream"
                );
                return Err(error);
            }
        };
        Ok(hash_string)
    }

    /// Calculates a digest hash from a data stream.
    fn calculate_hash_from_data_stream(
        &self,
        data_stream: &DataStreamReference,
    ) -> Result<String, ErrorTrace> {
        let mut hash_context: Box<dyn DigestHashContext> = match &self.digest_hash_type {
            DigestHashType::Md5 => Box::new(Md5Context::new()),
            DigestHashType::Sha1 => Box::new(Sha1Context::new()),
            DigestHashType::Sha224 => Box::new(Sha224Context::new()),
            DigestHashType::Sha256 => Box::new(Sha256Context::new()),
            DigestHashType::Sha512 => Box::new(Sha512Context::new()),
        };
        let mut data: [u8; Self::READ_BUFFER_SIZE] = [0; Self::READ_BUFFER_SIZE];

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
        let display_path: String = self.display_path.join_path_components(path_components);

        let number_of_data_forks: usize = match file_entry.get_number_of_data_forks() {
            Ok(number_of_data_forks) => number_of_data_forks,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to retrieve number of data forks of file entry: {}",
                        display_path
                    )
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
                        format!(
                            "Unable to retrieve data fork: {} of file entry: {}",
                            data_fork_index, display_path
                        )
                    );
                    return Err(error);
                }
            };
            let name: Option<VfsString> = data_fork.get_name();

            let display_path_and_name: String = match name.as_ref() {
                Some(name) => {
                    let escaped_name: String = self.display_path.escape_string(name);
                    format!("{}:{}", display_path, escaped_name)
                }
                None => display_path.clone(),
            };
            // TODO: add option for dfImageTools compatibility mode
            // if name == Some(String::from("WofCompressedData")) {
            //     continue;
            // }
            // TODO: create skip list
            let hash_string: String = if path_components.len() > 1
                && path_components[1] == VfsString::Ucs2(Ucs2String::from("$BadClus"))
                && name == Some(VfsString::Ucs2(Ucs2String::from("$Bad")))
            {
                String::from("N/A (skipped)")
            } else {
                match self.calculate_hash_from_data_fork(&data_fork) {
                    Ok(hash_string) => hash_string,
                    Err(mut error) => {
                        if self.stop_on_error {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!(
                                    "Unable to calculate hash of data stream: {}",
                                    display_path_and_name
                                )
                            );
                            return Err(error);
                        }
                        String::from("N/A (error)")
                    }
                }
            };
            println!(
                "{}\t{}{}",
                hash_string, file_system_display_path, display_path_and_name
            );
        }
        Ok(())
    }

    /// Calculates a digest hash from a scan node.
    fn calculate_hash_from_scan_node(&self, vfs_scan_node: &VfsScanNode) -> Result<(), ErrorTrace> {
        if vfs_scan_node.is_empty() {
            // Only process scan nodes that contain a file system.
            if !vfs_scan_node.is_file_system() {
                return Ok(());
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
                Some(parent_path) => match self.display_path.get_path(parent_path) {
                    Ok(path) => path,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to retrieve parent display path"
                        );
                        return Err(error);
                    }
                },
                None => String::new(),
            };
            for result in VfsFinder::new(&file_system) {
                match result {
                    Ok((file_entry, path_components)) => match self.calculate_hash_from_file_entry(
                        &file_entry,
                        &display_path,
                        &path_components,
                    ) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to calculate hash from file entry"
                            );
                            return Err(error);
                        }
                    },
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
                match self.calculate_hash_from_scan_node(sub_scan_node) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to calculate hash from sub scan node"
                        );
                        return Err(error);
                    }
                }
            }
        }
        Ok(())
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
                    println!("Unable to calculate hash from stdin\n{}", error);
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
                    println!("Unable to scan: {}\n{}", source, error);
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
                        println!("Unable to open file: {}\n{}", source, error);
                        return ExitCode::FAILURE;
                    }
                };
                let hash_string: String =
                    match hash_tool.calculate_hash_from_data_stream(&data_stream) {
                        Ok(hash) => hash,
                        Err(error) => {
                            if hash_tool.stop_on_error {
                                println!("Unable to calculate hash of: {}\n{}", source, error);
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
                        println!("Unable to calculate hash of: {}\n{}", source, error);
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
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let hash_tool: HashTool =
            HashTool::new(&DigestHashType::Md5, &DisplayPathType::Index, true);
        let md5: String = hash_tool.calculate_hash_from_data_stream(&data_stream)?;
        assert_eq!(md5, "f19106bcf25fa9cabc1b5ac91c726001");

        Ok(())
    }

    // TODO: add more tests
}
