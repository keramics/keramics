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

use std::collections::HashMap;
use std::sync::Arc;

use keramics_core::ErrorTrace;
use keramics_formats::PathComponent;

use super::component_table::ComponentTable;
use super::scan_object::ScanObject;
use super::signature::PathFilterSignature;

/// Path scan tree node.
#[derive(Debug)]
pub(super) struct ScanTreeNode {
    /// Component index.
    pub component_index: usize,

    /// Scan objects.
    pub scan_objects: HashMap<PathComponent, ScanObject>,

    /// Default scan object.
    pub default_scan_object: Box<ScanObject>,
}

impl ScanTreeNode {
    /// Creates a new scan tree node.
    pub fn new() -> Self {
        Self {
            component_index: 0,
            scan_objects: HashMap::new(),
            default_scan_object: Box::new(ScanObject::None),
        }
    }

    /// Builds a scan tree node.
    pub(super) fn build(
        &mut self,
        component_table: &ComponentTable,
        component_indexes_to_ignore: &[usize],
    ) -> Result<(), ErrorTrace> {
        self.component_index = match component_table.get_most_significant_component_index() {
            Some(component_index) => component_index,
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Unable to determine most significant component index"
                ));
            }
        };
        let signatures_in_node: Vec<Arc<PathFilterSignature>> =
            component_table.get_signatures_by_component_index(self.component_index);

        let mut remaining_signatures: Vec<Arc<PathFilterSignature>> = Vec::new();
        for path in component_table.signatures.iter() {
            if !signatures_in_node.contains(path) {
                remaining_signatures.push(Arc::clone(path));
            }
        }
        let mut sub_component_indexes_to_ignore: Vec<usize> = component_indexes_to_ignore.to_vec();
        sub_component_indexes_to_ignore.push(self.component_index);

        if let Some(component_group) = component_table.component_groups.get(&self.component_index) {
            for (group_index, (_, path_group)) in component_group.path_groups.iter().enumerate() {
                let number_of_signatures: usize = path_group.signatures.len();

                if number_of_signatures == 0 {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Invalid component group for component index: {} invalid path group: {} missing signatures",
                        self.component_index, group_index
                    )));
                }
                if number_of_signatures == 1 {
                    self.scan_objects.insert(
                        path_group.path_component.clone(),
                        ScanObject::Signature(Arc::clone(&path_group.signatures[0])),
                    );
                } else {
                    let mut sub_component_table: ComponentTable =
                        ComponentTable::new(&component_table.scan_tree_type);

                    sub_component_table
                        .fill(&path_group.signatures, &sub_component_indexes_to_ignore);
                    sub_component_table
                        .fill(&remaining_signatures, &sub_component_indexes_to_ignore);
                    sub_component_table.calculate_weights();

                    let mut sub_node: ScanTreeNode = ScanTreeNode::new();

                    match sub_node.build(&sub_component_table, &sub_component_indexes_to_ignore) {
                        Ok(_) => {}
                        Err(mut error) => {
                            error.add_frame(format!(
                                "Unable to build sub scan tree node for path group: {} and component index: {}",
                                group_index, self.component_index
                            ));
                            return Err(error);
                        }
                    }
                    self.scan_objects.insert(
                        path_group.path_component.clone(),
                        ScanObject::ScanTreeNode(sub_node),
                    );
                }
            }
        }
        let number_of_remaining_signatures: usize = remaining_signatures.len();

        if number_of_remaining_signatures == 1 {
            self.default_scan_object =
                Box::new(ScanObject::Signature(Arc::clone(&remaining_signatures[0])));
        } else if number_of_remaining_signatures > 1 {
            let mut sub_component_table: ComponentTable =
                ComponentTable::new(&component_table.scan_tree_type);

            sub_component_table.fill(&remaining_signatures, &sub_component_indexes_to_ignore);
            sub_component_table.calculate_weights();

            let mut sub_node: ScanTreeNode = ScanTreeNode::new();

            match sub_node.build(&sub_component_table, &sub_component_indexes_to_ignore) {
                Ok(_) => {}
                Err(mut error) => {
                    error.add_frame(format!(
                        "Unable to build sub scan tree node for remaining signatures and component index: {}",
                        self.component_index
                    ));
                    return Err(error);
                }
            }
            self.default_scan_object = Box::new(ScanObject::ScanTreeNode(sub_node));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use super::super::enums::ScanTreeType;
    use keramics_formats::Path;

    fn build_component_table(
        path_strings: &[&str],
        component_indexes_to_ignore: &[usize],
    ) -> ComponentTable {
        let mut component_table: ComponentTable = ComponentTable::new(&ScanTreeType::Prefix);

        let signatures: Vec<Arc<PathFilterSignature>> = path_strings
            .iter()
            .map(|path_string| Arc::new(PathFilterSignature::new(Path::from(*path_string), None)))
            .collect();

        component_table.fill(&signatures, component_indexes_to_ignore);
        component_table.calculate_weights();

        component_table
    }

    #[test]
    fn test_build_with_single_signature() -> Result<(), ErrorTrace> {
        let component_table: ComponentTable = build_component_table(&["/testdir1/testfile1"], &[]);

        let mut scan_tree_node: ScanTreeNode = ScanTreeNode::new();
        scan_tree_node.build(&component_table, &[])?;

        assert_eq!(scan_tree_node.component_index, 0);
        assert_eq!(scan_tree_node.scan_objects.len(), 1);
        assert!(matches!(
            scan_tree_node.scan_objects[&PathComponent::Root],
            ScanObject::Signature(_)
        ));
        assert!(matches!(
            *scan_tree_node.default_scan_object,
            ScanObject::None
        ));

        Ok(())
    }

    #[test]
    fn test_build_with_multiple_signatures_same_branch() -> Result<(), ErrorTrace> {
        let component_table: ComponentTable = build_component_table(
            &[
                "/Windows/SoftwareDistribution/DataStore/DataStore.edb",
                "/Windows/SoftwareDistribution/Downloads/scan.download",
            ],
            &[],
        );

        let mut scan_tree_node: ScanTreeNode = ScanTreeNode::new();
        scan_tree_node.build(&component_table, &[])?;

        // The two signatures differ at the component that is resolved by this
        // node, so both branches hold a single signature.
        assert_eq!(scan_tree_node.scan_objects.len(), 2);
        assert!(
            scan_tree_node
                .scan_objects
                .values()
                .all(|scan_object| matches!(scan_object, ScanObject::Signature(_)))
        );
        assert!(matches!(
            *scan_tree_node.default_scan_object,
            ScanObject::None
        ));

        Ok(())
    }

    #[test]
    fn test_build_with_sub_scan_tree_node() -> Result<(), ErrorTrace> {
        let component_table: ComponentTable = build_component_table(
            &[
                "/testdir1/subdir1/testfile1",
                "/testdir1/subdir1/testfile2",
                "/testdir1/subdir2/testfile3",
            ],
            &[],
        );

        let mut scan_tree_node: ScanTreeNode = ScanTreeNode::new();
        scan_tree_node.build(&component_table, &[])?;

        // The resolved component has a single path group holding multiple
        // signatures, so it is represented by a sub scan tree node.
        assert_eq!(scan_tree_node.scan_objects.len(), 1);
        assert!(
            scan_tree_node
                .scan_objects
                .values()
                .all(|scan_object| matches!(scan_object, ScanObject::ScanTreeNode(_)))
        );
        assert!(matches!(
            *scan_tree_node.default_scan_object,
            ScanObject::None
        ));

        Ok(())
    }

    #[test]
    fn test_build_with_multiple_signatures_different_groups() -> Result<(), ErrorTrace> {
        let component_table: ComponentTable = build_component_table(
            &[
                "/Windows/System32/winevt/Logs/Application.evtx",
                "/Library/Caches/com.apple.Safari",
            ],
            &[],
        );

        let mut scan_tree_node: ScanTreeNode = ScanTreeNode::new();
        scan_tree_node.build(&component_table, &[])?;

        // The two signatures differ at the first directory component, and each
        // branch holds a single signature.
        assert_eq!(scan_tree_node.scan_objects.len(), 2);
        assert!(
            scan_tree_node
                .scan_objects
                .values()
                .all(|scan_object| matches!(scan_object, ScanObject::Signature(_)))
        );
        assert!(matches!(
            *scan_tree_node.default_scan_object,
            ScanObject::None
        ));

        Ok(())
    }

    #[test]
    fn test_build_with_empty_component_table() {
        let component_table: ComponentTable = build_component_table(&[], &[]);

        let mut scan_tree_node: ScanTreeNode = ScanTreeNode::new();

        // Without signatures the most significant component index cannot be determined.
        let result: Result<(), ErrorTrace> = scan_tree_node.build(&component_table, &[]);
        assert!(result.is_err());
    }
}
