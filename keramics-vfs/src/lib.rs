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

mod context;
mod data_fork;
mod enums;
mod file_entry;
mod file_resolver;
mod file_system;
mod iterators;
mod location;
mod path;
mod string;
mod string_path;
mod types;

// Format specific back-ends.
mod apm;
mod ewf;
mod fake;
mod gpt;
mod mbr;
mod os;
mod qcow;
mod sparseimage;
mod udif;
mod vhd;
mod vhdx;

// Helpers.
mod finder;
mod resolver;
mod scanner;

pub use context::VfsContext;
pub use data_fork::VfsDataFork;
pub use enums::*;
pub use file_entry::VfsFileEntry;
pub use file_resolver::{VfsFileResolver, new_vfs_file_resolver};
pub use file_system::VfsFileSystem;
pub use finder::VfsFinder;
pub use location::{VfsLocation, new_os_vfs_location};
pub use path::VfsPath;
pub use resolver::VfsResolver;
pub use scanner::{VfsScanContext, VfsScanNode, VfsScanner, VfsScannerMediator};
pub use string::VfsString;
pub use types::*;

#[cfg(test)]
mod tests {
    pub fn get_test_data_path(path: &str) -> String {
        format!("../test_data/{}", path)
    }
}
