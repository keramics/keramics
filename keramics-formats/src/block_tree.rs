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

use std::rc::Rc;

use super::errors::InsertError;

/// Block tree node type.
#[derive(Clone, Default, PartialEq)]
enum BlockTreeNodeType {
    Branch,
    #[default]
    Leaf,
}

/// Block tree node.
struct BlockTreeNode<T> {
    /// Node type.
    pub node_type: BlockTreeNodeType,

    /// Offset of the block.
    pub offset: u64,

    /// Size of the block represented by a sub node or (leaf) value.
    pub element_size: u64,

    /// Sub nodes.
    pub sub_nodes: Vec<Option<BlockTreeNode<T>>>,

    /// Values.
    pub values: Vec<Option<Rc<T>>>,
}

impl<T> BlockTreeNode<T> {
    /// Creates a new block tree node.
    fn new(node_type: &BlockTreeNodeType, offset: u64, element_size: u64) -> Self {
        BlockTreeNode {
            node_type: node_type.clone(),
            offset: offset,
            element_size: element_size,
            sub_nodes: Vec::new(),
            values: Vec::new(),
        }
    }

    /// Inserts a (leaf) value.
    fn insert_value(
        &mut self,
        elements_per_node: u64,
        leaf_value_size: u64,
        offset: u64,
        size: u64,
        value: Rc<T>,
    ) -> Result<(), InsertError> {
        if self.node_type == BlockTreeNodeType::Branch {
            if self.sub_nodes.len() == 0 {
                self.sub_nodes = (0..elements_per_node).map(|_| None).collect();
            }
            let number_of_sub_nodex: u64 = size.div_ceil(self.element_size);
            let first_sub_node_index: u64 = (offset - self.offset) / self.element_size;
            let last_sub_node_index: u64 = first_sub_node_index + number_of_sub_nodex;

            let mut sub_node_element_size: u64 = leaf_value_size;

            while self.element_size / sub_node_element_size > elements_per_node {
                sub_node_element_size *= elements_per_node;
            }
            let sub_node_type: BlockTreeNodeType = if sub_node_element_size <= size {
                BlockTreeNodeType::Leaf
            } else {
                BlockTreeNodeType::Branch
            };
            let mut sub_node_offset: u64 = self.offset + (first_sub_node_index * self.element_size);

            for sub_node_index in first_sub_node_index..last_sub_node_index {
                if self.sub_nodes[sub_node_index as usize].is_none() {
                    let sub_node: BlockTreeNode<T> =
                        BlockTreeNode::new(&sub_node_type, sub_node_offset, sub_node_element_size);
                    self.sub_nodes[sub_node_index as usize] = Some(sub_node);
                }
                let sub_node: &mut BlockTreeNode<T> =
                    &mut self.sub_nodes[sub_node_index as usize].as_mut().unwrap();
                sub_node.insert_value(
                    elements_per_node,
                    leaf_value_size,
                    offset,
                    size,
                    value.clone(),
                )?;
                sub_node_offset += self.element_size;
            }
        } else {
            if size % self.element_size != 0 {
                return Err(InsertError::new(format!(
                    "Size: {} not a multitude of node element size: {}",
                    size, self.element_size
                )));
            }
            if self.values.len() == 0 {
                self.values = (0..elements_per_node).map(|_| None).collect();
            }
            let number_of_values: u64 = size / self.element_size;
            let first_value_index: u64 = (offset - self.offset) / self.element_size;
            let last_value_index: u64 = first_value_index + number_of_values;

            for value_index in first_value_index..last_value_index {
                if self.values[value_index as usize].is_some() {
                    return Err(InsertError::new(format!("Leaf value already set")));
                }
                self.values[value_index as usize] = Some(value.clone());
            }
        }
        Ok(())
    }
}

/// Block tree.
pub struct BlockTree<T> {
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
            data_size: data_size,
            elements_per_node: elements_per_node,
            leaf_value_size: leaf_value_size,
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
    pub fn get_value(&self, offset: u64) -> Option<&T> {
        if self.root_node.is_none() {
            return None;
        }
        let mut node: &BlockTreeNode<T> = self.root_node.as_ref().unwrap();

        while node.node_type == BlockTreeNodeType::Branch {
            let sub_node_index: u64 = (offset - node.offset) / node.element_size;

            if node.sub_nodes[sub_node_index as usize].is_none() {
                return None;
            }
            node = node.sub_nodes[sub_node_index as usize].as_ref().unwrap();
        }
        let value_index: usize = ((offset - node.offset) / node.element_size) as usize;

        if value_index >= node.values.len() || node.values[value_index].is_none() {
            return None;
        }
        Some(node.values[value_index].as_ref().unwrap())
    }

    /// Inserts a (leaf) value.
    pub fn insert_value(&mut self, offset: u64, size: u64, value: T) -> Result<(), InsertError> {
        if offset + size > self.data_size {
            return Err(InsertError::new(format!(
                "Range: {} - {} exceeds data size: {}",
                offset,
                offset + size,
                self.data_size
            )));
        }
        if offset % self.leaf_value_size != 0 {
            return Err(InsertError::new(format!(
                "Offset: {} not a multitude of leaf value size: {}",
                offset, self.leaf_value_size
            )));
        }
        if size % self.leaf_value_size != 0 {
            return Err(InsertError::new(format!(
                "Size: {} not a multitude of leaf value size: {}",
                size, self.leaf_value_size
            )));
        }
        if self.root_node.is_none() {
            self.create_root_node(size);
        }
        let root_node: &mut BlockTreeNode<T> = &mut self.root_node.as_mut().unwrap();
        root_node.insert_value(
            self.elements_per_node,
            self.leaf_value_size,
            offset,
            size,
            Rc::new(value),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_get_value() -> Result<(), InsertError> {
        let mut test_tree: BlockTree<u32> = BlockTree::new(1048576, 256, 512);

        let test_leaf_value: u32 = 0x12345678;
        test_tree.insert_value(131072, 512, test_leaf_value)?;

        let value: Option<&u32> = test_tree.get_value(0);
        assert!(value.is_none());

        let value: Option<&u32> = test_tree.get_value(131328);
        assert!(value.is_some());

        let leaf_value: &u32 = value.unwrap();
        assert_eq!(*leaf_value, 0x12345678);

        Ok(())
    }

    #[test]
    fn test_tree_insert_value_with_leaf_size() -> Result<(), InsertError> {
        let mut test_tree: BlockTree<u32> = BlockTree::new(1048576, 256, 512);

        let test_leaf_value: u32 = 0x12345678;
        test_tree.insert_value(131072, 512, test_leaf_value)?;

        let test_node: &BlockTreeNode<u32> = test_tree.root_node.as_ref().unwrap();
        assert!(test_node.node_type == BlockTreeNodeType::Branch);
        assert_eq!(test_node.offset, 0);
        assert_eq!(test_node.element_size, 131072);
        assert_eq!(test_node.sub_nodes.len(), 8);
        assert_eq!(test_node.values.len(), 0);

        let test_node: &BlockTreeNode<u32> = test_node.sub_nodes[1].as_ref().unwrap();
        assert!(test_node.node_type == BlockTreeNodeType::Leaf);
        assert_eq!(test_node.offset, 131072);
        assert_eq!(test_node.element_size, 512);
        assert_eq!(test_node.sub_nodes.len(), 0);
        assert_eq!(test_node.values.len(), 256);

        Ok(())
    }

    #[test]
    fn test_tree_insert_value_with_element_size() -> Result<(), InsertError> {
        let mut test_tree: BlockTree<u32> = BlockTree::new(1048576, 256, 512);

        let test_leaf_value: u32 = 0x12345678;
        test_tree.insert_value(131072, 131072, test_leaf_value)?;

        let test_node: &BlockTreeNode<u32> = test_tree.root_node.as_ref().unwrap();
        assert!(test_node.node_type == BlockTreeNodeType::Leaf);
        assert_eq!(test_node.offset, 0);
        assert_eq!(test_node.element_size, 131072);
        assert_eq!(test_node.sub_nodes.len(), 0);
        assert_eq!(test_node.values.len(), 8);

        Ok(())
    }

    #[test]
    fn test_tree_insert_value_with_range_outside_tree() {
        let mut test_tree: BlockTree<u32> = BlockTree::new(1048576, 256, 512);

        let test_leaf_value: u32 = 0x12345678;
        let result = test_tree.insert_value(983040, 131072, test_leaf_value);
        assert!(result.is_err());
    }

    #[test]
    fn test_tree_insert_value_with_unsupported_offset() {
        let mut test_tree: BlockTree<u32> = BlockTree::new(1048576, 256, 512);

        let test_leaf_value: u32 = 0x12345678;
        let result = test_tree.insert_value(131000, 512, test_leaf_value);
        assert!(result.is_err());
    }

    #[test]
    fn test_tree_insert_value_with_unsupported_size() {
        let mut test_tree: BlockTree<u32> = BlockTree::new(1048576, 256, 512);

        let test_leaf_value: u32 = 0x12345678;
        let result = test_tree.insert_value(131072, 500, test_leaf_value);
        assert!(result.is_err());
    }
}
