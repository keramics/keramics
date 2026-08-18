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

use std::iter::repeat_with;
use std::sync::Arc;

use keramics_core::ErrorTrace;

/// Block tree node type.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum BlockTreeNodeType {
    Branch,
    Leaf,
}

pub(crate) enum BlockTreeNodeElements<T> {
    /// Branch node elements (sub nodes).
    Branch(Box<[Option<BlockTreeNode<T>>]>),

    /// Leaf node elements (values).
    Leaf(Box<[Option<Arc<T>>]>),
}

impl<T> BlockTreeNodeElements<T> {
    /// Retrieves the number of elements.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        match self {
            BlockTreeNodeElements::Branch(sub_nodes) => sub_nodes.len(),
            BlockTreeNodeElements::Leaf(values) => values.len(),
        }
    }
}

/// Block tree node.
pub(crate) struct BlockTreeNode<T> {
    /// Offset of the block.
    pub offset: u64,

    /// Size of the block represented by a sub node or (leaf) value.
    pub element_size: u64,

    /// Elements.
    pub elements: BlockTreeNodeElements<T>,
}

impl<T> BlockTreeNode<T> {
    /// Creates a new block tree node.
    pub fn new(
        node_type: &BlockTreeNodeType,
        offset: u64,
        element_size: u64,
        elements_per_node: u64,
    ) -> Self {
        let elements: BlockTreeNodeElements<T> = match node_type {
            BlockTreeNodeType::Branch => BlockTreeNodeElements::Branch(
                repeat_with(|| None)
                    .take(elements_per_node as usize)
                    .collect::<Box<[Option<BlockTreeNode<T>>]>>(),
            ),
            BlockTreeNodeType::Leaf => BlockTreeNodeElements::Leaf(
                repeat_with(|| None)
                    .take(elements_per_node as usize)
                    .collect::<Box<[Option<Arc<T>>]>>(),
            ),
        };
        BlockTreeNode {
            offset,
            element_size,
            elements,
        }
    }

    /// Retrieves a specific sub node.
    pub(super) fn get_sub_node(&self, index: usize) -> Option<&BlockTreeNode<T>> {
        match &self.elements {
            BlockTreeNodeElements::Branch(sub_nodes) => match sub_nodes.get(index) {
                Some(Some(node)) => Some(node),
                _ => None,
            },
            _ => None,
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
    ) -> Result<(), ErrorTrace> {
        match &mut self.elements {
            BlockTreeNodeElements::Branch(sub_nodes) => {
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
                let mut sub_node_offset: u64 =
                    self.offset + (first_sub_node_index * self.element_size);

                for sub_node_index in first_sub_node_index..last_sub_node_index {
                    if sub_nodes[sub_node_index as usize].is_none() {
                        let sub_node: BlockTreeNode<T> = BlockTreeNode::new(
                            &sub_node_type,
                            sub_node_offset,
                            sub_node_element_size,
                            elements_per_node,
                        );
                        sub_nodes[sub_node_index as usize] = Some(sub_node);
                    }
                    let sub_node: &mut BlockTreeNode<T> =
                        match sub_nodes[sub_node_index as usize].as_mut() {
                            Some(node) => node,
                            None => {
                                return Err(keramics_core::error_trace_new!(format!(
                                    "Unable to obtain mutable reference to sub node: {}",
                                    sub_node_index
                                )));
                            }
                        };
                    match sub_node.insert_value(
                        elements_per_node,
                        leaf_value_size,
                        offset,
                        size,
                        value.clone(),
                    ) {
                        Ok(_) => {}
                        Err(error) => {
                            return Err(keramics_core::error_trace_new_with_error!(
                                format!("Unable to insert value into sub node: {}", sub_node_index),
                                error
                            ));
                        }
                    }
                    sub_node_offset += self.element_size;
                }
            }
            BlockTreeNodeElements::Leaf(values) => {
                if !size.is_multiple_of(self.element_size) {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Size: {} not a multitude of node element size: {}",
                        size, self.element_size
                    )));
                }
                let number_of_values: u64 = size / self.element_size;
                let first_value_index: u64 = (offset - self.offset) / self.element_size;
                let last_value_index: u64 = first_value_index + number_of_values;

                for value_index in first_value_index..last_value_index {
                    if values[value_index as usize].is_some() {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Leaf value: {} already set",
                            value_index
                        )));
                    }
                    values[value_index as usize] = Some(value.clone());
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_value_with_leaf_size() -> Result<(), ErrorTrace> {
        let mut test_node: BlockTreeNode<u32> =
            BlockTreeNode::<u32>::new(&BlockTreeNodeType::Leaf, 0, 512, 256);

        test_node.insert_value(256, 512, 0, 512, Arc::new(42))?;

        assert_eq!(test_node.elements.len(), 256);

        Ok(())
    }
}
