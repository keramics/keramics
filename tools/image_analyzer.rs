/* Copyright 2024 Joachim Metz <joachim.metz@gmail.com>
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
use std::rc::Rc;

use clap::Parser;

use keramics::vfs::{VfsPath, VfsPathType, VfsScanContext, VfsScanNode, VfsScanner};

#[derive(Parser)]
#[command(version, about = "Analyzes the contents of a storage media image", long_about = None)]
struct CommandLineArguments {
    /// Path of the source file
    source: std::path::PathBuf,
}

fn print_scan_node(scan_node: &VfsScanNode, depth: usize) {
    let indentation: String = vec![" "; depth * 4].join("");
    let format_identifier: &str = match scan_node.path.get_path_type() {
        VfsPathType::Apm => "APM",
        VfsPathType::Ext => "EXT",
        VfsPathType::Fake => "FAKE",
        VfsPathType::Gpt => "GPT",
        VfsPathType::Mbr => "MBR",
        VfsPathType::Os => "OS",
        VfsPathType::Qcow => "QCOW",
        VfsPathType::Vhd => "VHD",
        VfsPathType::Vhdx => "VHDX",
    };
    println!(
        "{}{}: location: {}",
        indentation,
        format_identifier,
        scan_node.path.get_location()
    );

    for sub_scan_node in scan_node.sub_nodes.iter() {
        print_scan_node(sub_scan_node, depth + 1);
    }
}

fn main() -> ExitCode {
    let arguments = CommandLineArguments::parse();

    let source: &str = match arguments.source.to_str() {
        Some(value) => value,
        None => {
            println!("Missing source");
            return ExitCode::FAILURE;
        }
    };
    let vfs_path: VfsPath = VfsPath::new(VfsPathType::Os, source, None);

    let mut vfs_scanner: VfsScanner = VfsScanner::new();
    match vfs_scanner.build() {
        Ok(_) => {}
        Err(error) => {
            println!("{}", error);
            return ExitCode::FAILURE;
        }
    };
    let mut vfs_scan_context: VfsScanContext = VfsScanContext::new();
    match vfs_scanner.scan(&mut vfs_scan_context, &Rc::new(vfs_path)) {
        Ok(_) => {}
        Err(error) => {
            println!("Unable to scan: {} with error: {}", source, error);
            return ExitCode::FAILURE;
        }
    };
    // TODO: print source type.

    match vfs_scan_context.root_node {
        Some(scan_node) => print_scan_node(&scan_node, 0),
        None => {}
    };
    ExitCode::SUCCESS
}
