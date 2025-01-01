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

mod block_allocation_table;
mod block_range;
mod constants;
mod enums;
mod file;
mod file_header;
mod image;
mod image_header;
mod layer;
mod metadata_table;
mod metadata_table_entry;
mod metadata_table_header;
mod parent_locator;
mod parent_locator_entry;
mod parent_locator_header;
mod region_table;
mod region_table_entry;
mod region_table_header;
mod sector_bitmap;

pub use enums::VhdxDiskType;
pub use file::VhdxFile;
pub use image::VhdxImage;
pub use layer::VhdxLayer;
