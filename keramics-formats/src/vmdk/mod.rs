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
mod compressed_grain_header;
pub(crate) mod constants;
mod descriptor_extent;
mod descriptor_storage;
mod enums;
mod extent_file;
mod image;
mod image_layer;
mod sector_table;
mod sparse_cowd_file;
mod sparse_cowd_file_header;
mod sparse_file;
mod sparse_file_header;

pub use enums::{VmdkCompressionMethod, VmdkDiskType};
pub use image::VmdkImage;
pub use image_layer::VmdkImageLayer;
