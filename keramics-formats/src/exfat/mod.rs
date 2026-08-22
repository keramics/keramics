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

mod allocation_bitmap_record;
mod block_allocation_table;
mod block_range;
mod block_reader;
mod block_stream;
mod boot_record;
mod case_folding_mappings_record;
mod constants;
mod data_stream_record;
mod directory_entries;
mod directory_entry;
mod directory_entry_type;
mod file_entries;
mod file_entry;
mod file_entry_record;
mod file_name_record;
mod file_system;
mod volume_label_record;

pub use file_entry::ExFatFileEntry;
pub use file_system::ExFatFileSystem;
