/* Copyright 2024-2026 Joachim Metz <joachim.metz@gmail.com>
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

mod block_stream;
mod block_tree;
mod block_tree_node;
mod enums;
mod fake_file_resolver;
mod file_entries_iterator;
mod file_resolver;
mod indexed_hash_map;
pub mod lru_cache;
mod macros;
mod os_file_resolver;
mod path;
mod path_component;
mod range_stream;
mod scanner;
mod traits;
mod util;

// Data formats used in other formats.
pub mod cdsaencr;
mod decmpfs;
mod plist;
mod xml;

// Storage media image formats
pub mod ewf;
pub mod pdi;
pub mod qcow;
pub mod sparsebundle;
pub mod sparseimage;
pub mod splitraw;
pub mod udif;
pub mod vhd;
pub mod vhdx;
pub mod vmdk;

// Volume system formats
pub mod apm;
pub mod bsdlabel;
pub mod gpt;
pub mod linuxlvm;
pub mod mbr;

// Hybrid volume and file system formats
pub mod apfs;

// File system formats
pub mod exfat;
pub mod ext;
pub mod fat;
pub mod hfs;
pub mod ntfs;

pub use enums::FormatIdentifier;
pub use file_entries_iterator::FileEntriesIterator;
pub use file_resolver::{FileResolver, FileResolverReference};
pub use os_file_resolver::{OsFileResolver, open_os_file_resolver};
pub use path::Path;
pub use path_component::PathComponent;
pub use range_stream::RangeStream;
pub use scanner::FormatScanner;
pub use traits::FileEntryIterator;

#[cfg(test)]
mod tests {
    pub fn get_test_data_path(path: &str) -> String {
        format!("../test_data/{}", path)
    }
}
