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
mod btree_entry;
mod btree_entry_fixed_size;
mod btree_entry_variable_size;
mod btree_footer;
mod btree_node;
mod btree_node_header;
mod change_information;
mod checkpoint_map;
mod checkpoint_map_entry;
pub mod constants;
mod container;
mod container_superblock;
mod encryption_state;
mod object_checksum;
mod object_header;
mod object_map;
mod object_map_key;
mod object_map_tree;
mod object_map_value;
mod volume;
mod volume_superblock;
mod volumes;

pub use container::ApfsContainer;
pub use volume::ApfsVolume;
