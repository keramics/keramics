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

mod block_range;
mod block_reader;
mod block_stream;
mod block_table;
mod block_table_entry;
mod block_table_header;
mod block_table_reader;
pub(crate) mod constants;
mod enums;
mod file;
mod file_footer;
mod image;
mod resource_descriptor;
mod resource_fork_header;
mod resource_map;
mod resource_map_entry;
mod resource_map_header;
mod resource_map_item;
mod resource_map_value;
mod segment_file;
mod segment_range;
mod segments_block_reader;
mod segments_block_stream;

pub use enums::UdifCompressionMethod;
pub use file::UdifFile;
pub use image::UdifImage;
