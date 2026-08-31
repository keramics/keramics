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

mod attribute;
mod attributes_table;
mod attributes_table_entry;
mod attributes_table_header;
mod attributes_tree;
mod attributes_tree_leaf_entry;
mod attributes_tree_local_value;
mod attributes_tree_remote_value;
mod block_free_region;
mod block_reader;
mod block_stream;
mod block_tree_branch_entry;
mod block_tree_branch_header;
mod block_tree_leaf_header;
mod btree_node;
mod btree_node_header_32bit;
mod btree_node_header_64bit;
pub mod constants;
mod directory_entry;
mod directory_list;
mod directory_list_element;
mod directory_list_element_entry_v2;
mod directory_list_element_footer_v2;
mod directory_list_element_header_v2;
mod directory_list_element_header_v3;
mod directory_list_element_unused_entry_v2;
mod directory_table;
mod directory_table_entry_v1;
mod directory_table_entry_v2;
mod directory_table_header_v1;
mod directory_table_header_v2_32bit;
mod directory_table_header_v2_64bit;
mod directory_tree_leaf_entry;
mod extended_attribute;
mod extended_attributes;
mod extent_list;
mod extent_tree;
mod extent_tree_branch_header;
mod extent_tree_branch_key;
mod extent_tree_branch_value;
mod features;
mod file_entries;
mod file_entry;
mod file_system;
mod file_system_block;
mod file_system_block_header_v1;
mod file_system_block_header_v3;
mod inode;
mod inode_information;
mod inode_tree;
mod inode_tree_branch_key;
mod inode_tree_branch_value;
mod inode_tree_leaf_record;
mod inode_v1;
mod inode_v2;
mod inode_v3;
mod packed_extent;
mod superblock;

pub use extended_attribute::XfsExtendedAttribute;
pub use file_entry::XfsFileEntry;
pub use file_system::XfsFileSystem;
