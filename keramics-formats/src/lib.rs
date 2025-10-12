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

mod block_tree;
mod enums;
mod errors;
mod fake_file_resolver;
mod file_resolver;
mod lru_cache;
mod os_file_resolver;
mod path_component;
mod scanner;

// Data formats used in other formats.
mod plist;

// Storage media image formats
pub mod ewf;
pub mod qcow;
pub mod sparsebundle;
pub mod sparseimage;
pub mod udif;
pub mod vhd;
pub mod vhdx;

// Volume system formats
pub mod apm;
pub mod gpt;
pub mod mbr;

// File system formats
pub mod ext;
pub mod ntfs;

pub use enums::FormatIdentifier;
pub use file_resolver::{FileResolver, FileResolverReference};
pub use os_file_resolver::{OsFileResolver, open_os_file_resolver};
pub use path_component::PathComponent;
pub use scanner::FormatScanner;
