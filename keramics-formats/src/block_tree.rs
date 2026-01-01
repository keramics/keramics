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

use std::sync::Arc;

use keramics_core::ErrorTrace;

use super::block_tree_node::{BlockTreeNode, BlockTreeNodeType};

/// Block tree.
pub(crate) struct BlockTree<T> {
    /// Size of the data represented by the tree.
    data_size: u64,

    /// Number of elements per nodes.
    elements_per_node: u64,

    /// Size of a leaf value.
    leaf_value_size: u64,

    /// Root node.
    root_node: Option<BlockTreeNode<T>>,
}

impl<T> BlockTree<T> {
    /// Creates a new block tree.
    pub fn new(data_size: u64, elements_per_node: u64, leaf_value_size: u64) -> Self {
        Self {
            data_size,
            elements_per_node,
            leaf_value_size,
            root_node: None,
        }
    }

    /// Creates the root node.
    fn create_root_node(&mut self, size: u64) {
        let mut element_size: u64 = self.leaf_value_size;

        if self.elements_per_node * element_size > self.leaf_value_size {
            while self.data_size / element_size > self.elements_per_node {
                element_size *= self.elements_per_node;
            }
        }
        let elements_per_node: u64 = self.data_size.div_ceil(element_size);

        let node_type: BlockTreeNodeType = if element_size <= size {
            BlockTreeNodeType::Leaf
        } else {
            BlockTreeNodeType::Branch
        };
        let mut root_node: BlockTreeNode<T> = BlockTreeNode::<T>::new(&node_type, 0, element_size);

        match node_type {
            BlockTreeNodeType::Branch => {
                root_node.sub_nodes = (0..elements_per_node).map(|_| None).collect();
            }
            BlockTreeNodeType::Leaf => {
                root_node.values = (0..elements_per_node).map(|_| None).collect();
            }
        };
        self.root_node = Some(root_node);
    }

    /// Retrieves a (leaf) value.
    pub fn get_value(&self, offset: u64) -> Result<Option<&T>, ErrorTrace> {
        if self.root_node.is_none() {
            return Ok(None);
        }
        let mut node: &BlockTreeNode<T> = match self.root_node.as_ref() {
            Some(node) => node,
            None => {
                return Err(keramics_core::error_trace_new!("Missing root node"));
            }
        };
        while node.node_type == BlockTreeNodeType::Branch {
            let sub_node_index: u64 = (offset - node.offset) / node.element_size;

            if node.sub_nodes[sub_node_index as usize].is_none() {
                return Ok(None);
            }
            node = match node.sub_nodes[sub_node_index as usize].as_ref() {
                Some(node) => node,
                None => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Missing sub node: {}",
                        sub_node_index
                    )));
                }
            };
        }
        let value_index: usize = ((offset - node.offset) / node.element_size) as usize;

        if value_index >= node.values.len() || node.values[value_index].is_none() {
            return Ok(None);
        }
        match node.values[value_index].as_ref() {
            Some(node) => Ok(Some(node)),
            None => Err(keramics_core::error_trace_new!(format!(
                "Missing value: {}",
                value_index
            ))),
        }
    }

    /// Inserts a (leaf) value.
    pub fn insert_value(&mut self, offset: u64, size: u64, value: T) -> Result<(), ErrorTrace> {
        if offset + size > self.data_size {
            return Err(keramics_core::error_trace_new!(format!(
                "Range: {} - {} exceeds data size: {}",
                offset,
                offset + size,
                self.data_size
            )));
        }
        if !offset.is_multiple_of(self.leaf_value_size) {
            return Err(keramics_core::error_trace_new!(format!(
                "Offset: {} not a multitude of leaf value size: {}",
                offset, self.leaf_value_size
            )));
        }
        if !size.is_multiple_of(self.leaf_value_size) {
            return Err(keramics_core::error_trace_new!(format!(
                "Size: {} not a multitude of leaf value size: {}",
                size, self.leaf_value_size
            )));
        }
        if self.root_node.is_none() {
            self.create_root_node(size);
        }
        let root_node: &mut BlockTreeNode<T> = match self.root_node.as_mut() {
            Some(node) => node,
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Unable to obtain mutable reference to root node"
                ));
            }
        };
        match root_node.insert_value(
            self.elements_per_node,
            self.leaf_value_size,
            offset,
            size,
            Arc::new(value),
        ) {
            Ok(_) => Ok(()),
            Err(error) => Err(keramics_core::error_trace_new_with_error!(
                "Unable to insert value into root node",
                error
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_value() -> Result<(), ErrorTrace> {
        let mut test_tree: BlockTree<u32> = BlockTree::new(1048576, 256, 512);

        test_tree.insert_value(131072, 512, 0x12345678)?;

        let value: Option<&u32> = test_tree.get_value(0)?;
        assert_eq!(value, None);

        let value: Option<&u32> = test_tree.get_value(131328)?;
        assert_eq!(value, Some(&0x12345678));

        Ok(())
    }

    #[test]
    fn test_insert_value_with_leaf_size() -> Result<(), ErrorTrace> {
        let mut test_tree: BlockTree<u32> = BlockTree::new(1048576, 256, 512);

        test_tree.insert_value(131072, 512, 0x12345678)?;

        let test_node: &BlockTreeNode<u32> = test_tree.root_node.as_ref().unwrap();
        assert_eq!(test_node.node_type, BlockTreeNodeType::Branch);
        assert_eq!(test_node.offset, 0);
        assert_eq!(test_node.element_size, 131072);
        assert_eq!(test_node.sub_nodes.len(), 8);
        assert_eq!(test_node.values.len(), 0);

        let test_node: &BlockTreeNode<u32> = test_node.sub_nodes[1].as_ref().unwrap();
        assert_eq!(test_node.node_type, BlockTreeNodeType::Leaf);
        assert_eq!(test_node.offset, 131072);
        assert_eq!(test_node.element_size, 512);
        assert_eq!(test_node.sub_nodes.len(), 0);
        assert_eq!(test_node.values.len(), 256);

        Ok(())
    }

    #[test]
    fn test_insert_value_with_element_size() -> Result<(), ErrorTrace> {
        let mut test_tree: BlockTree<u32> = BlockTree::new(1048576, 256, 512);

        let test_leaf_value: u32 = 0x12345678;
        test_tree.insert_value(131072, 131072, test_leaf_value)?;

        let test_node: &BlockTreeNode<u32> = test_tree.root_node.as_ref().unwrap();
        assert_eq!(test_node.node_type, BlockTreeNodeType::Leaf);
        assert_eq!(test_node.offset, 0);
        assert_eq!(test_node.element_size, 131072);
        assert_eq!(test_node.sub_nodes.len(), 0);
        assert_eq!(test_node.values.len(), 8);

        Ok(())
    }

    #[test]
    fn test_insert_value_with_range_outside_tree() {
        let mut test_tree: BlockTree<u32> = BlockTree::new(1048576, 256, 512);

        let test_leaf_value: u32 = 0x12345678;
        let result = test_tree.insert_value(983040, 131072, test_leaf_value);
        assert!(result.is_err());
    }

    #[test]
    fn test_insert_value_with_unsupported_offset() {
        let mut test_tree: BlockTree<u32> = BlockTree::new(1048576, 256, 512);

        let test_leaf_value: u32 = 0x12345678;
        let result = test_tree.insert_value(131000, 512, test_leaf_value);
        assert!(result.is_err());
    }

    #[test]
    fn test_insert_value_with_unsupported_size() {
        let mut test_tree: BlockTree<u32> = BlockTree::new(1048576, 256, 512);

        let test_leaf_value: u32 = 0x12345678;
        let result = test_tree.insert_value(131072, 500, test_leaf_value);
        assert!(result.is_err());
    }
}
