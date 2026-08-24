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
mod block_reader;
mod block_stream;
pub(crate) mod constants;
mod enums;
mod extent_file;
mod image;
mod image_extent;
mod image_layer;
mod segment_descriptor;
mod segment_file_descriptor;
mod snapshot_descriptor;
mod sparse_file;
mod sparse_file_header;

pub use enums::PdiSegmentFileType;
pub use image::PdiImage;
pub use image_layer::PdiImageLayer;
pub use segment_descriptor::PdiSegmentDescriptor;
pub use segment_file_descriptor::PdiSegmentFileDescriptor;
pub use snapshot_descriptor::PdiSnapshotDescriptor;
