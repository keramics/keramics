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

mod block_allocation_table;
mod block_range;
mod block_stream;
mod boot_record;
mod boot_record_fat12;
mod boot_record_fat32;
pub mod constants;
mod directory_entries;
mod directory_entry;
mod directory_entry_type;
mod enums;
mod file_entries;
mod file_entry;
mod file_system;
mod long_name_directory_entry;
mod short_name_directory_entry;
mod short_name_directory_entry_fat12;
mod short_name_directory_entry_fat32;
mod string;

pub use enums::FatFormat;
pub use file_entry::FatFileEntry;
pub use file_system::FatFileSystem;
pub use string::FatString;
