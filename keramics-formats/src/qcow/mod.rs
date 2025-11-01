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

mod block_range;
mod cluster_table;
mod constants;
mod enums;
mod file;
mod file_header;
mod file_header_v1;
mod file_header_v2;
mod file_header_v3;
mod image;

pub use enums::{QcowCompressionMethod, QcowEncryptionMethod};
pub use file::QcowFile;
pub use image::{QcowImage, QcowImageLayer};
