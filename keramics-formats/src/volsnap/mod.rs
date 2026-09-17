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

mod backing_volume;
mod catalog_block;
mod catalog_block_header;
mod catalog_entry_type2;
mod catalog_entry_type3;
pub mod constants;
mod shadow_copy;
mod snapshot;
mod snapshots;
mod store_block;
mod store_block_header;
mod store_metadata;
mod volume_header;

pub use backing_volume::VolsnapBackingVolume;
pub use snapshot::VolsnapSnapshot;
