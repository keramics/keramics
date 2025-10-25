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

use std::sync::Arc;

use super::errors::InsertError;

/// Block tree node type.
#[derive(Clone, PartialEq)]
pub(crate) enum BlockTreeNodeType {
    Branch,
    Leaf,
}

/// Block tree node.
pub(crate) struct BlockTreeNode<T> {
    /// Node type.
    pub node_type: BlockTreeNodeType,

    /// Offset of the block.
    pub offset: u64,

    /// Size of the block represented by a sub node or (leaf) value.
    pub element_size: u64,

    /// Sub nodes.
    pub sub_nodes: Vec<Option<BlockTreeNode<T>>>,

    /// Values.
    pub values: Vec<Option<Arc<T>>>,
}

impl<T> BlockTreeNode<T> {
    /// Creates a new block tree node.
    pub fn new(node_type: &BlockTreeNodeType, offset: u64, element_size: u64) -> Self {
        BlockTreeNode {
            node_type: node_type.clone(),
            offset: offset,
            element_size: element_size,
            sub_nodes: Vec::new(),
            values: Vec::new(),
        }
    }

    /// Inserts a (leaf) value.
    pub fn insert_value(
        &mut self,
        elements_per_node: u64,
        leaf_value_size: u64,
        offset: u64,
        size: u64,
        value: Arc<T>,
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

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: add tests
}
