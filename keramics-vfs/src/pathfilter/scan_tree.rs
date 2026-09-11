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

use keramics_core::ErrorTrace;
use keramics_formats::Path;

use super::component_table::ComponentTable;
use super::enums::ScanTreeType;
use super::scan_object::ScanObject;
use super::scan_tree_node::ScanTreeNode;
use super::signature::PathFilterSignature;

/// Path scan tree.
pub(super) struct ScanTree {
    /// Scan tree type.
    scan_tree_type: ScanTreeType,

    /// Root scan object.
    root_scan_object: ScanObject,
}

impl ScanTree {
    /// Creates a new scan tree.
    pub fn new(scan_tree_type: ScanTreeType) -> Self {
        Self {
            scan_tree_type,
            root_scan_object: ScanObject::None,
        }
    }

    /// Builds the scan tree.
    pub fn build(&mut self, signatures: &[Arc<PathFilterSignature>]) -> Result<(), ErrorTrace> {
        let mut component_table: ComponentTable = ComponentTable::new(&self.scan_tree_type);

        let component_indexes_to_ignore: Vec<usize> = Vec::new();
        component_table.fill(signatures, &component_indexes_to_ignore);

        if !component_table.is_empty() {
            component_table.calculate_weights();

            let mut root_node: ScanTreeNode = ScanTreeNode::new();

            match root_node.build(&component_table, &component_indexes_to_ignore) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to build root node");
                    return Err(error);
                }
            }
            self.root_scan_object = ScanObject::ScanTreeNode(root_node);
        }
        Ok(())
    }

    /// Scans a path for a matching scan object.
    pub fn scan_path(&self, path: &Path) -> Option<&PathFilterSignature> {
        let mut scan_object: &ScanObject = &self.root_scan_object;

        loop {
            match scan_object {
                ScanObject::None => break,
                ScanObject::ScanTreeNode(scan_tree_node) => {
                    match path.components.get(scan_tree_node.component_index) {
                        Some(path_component) => {
                            // TODO: handle case folding

                            scan_object = match scan_tree_node.scan_objects.get(path_component) {
                                Some(sub_scan_object) => sub_scan_object,
                                None => &scan_tree_node.default_scan_object,
                            };
                        }
                        None => {
                            scan_object = &scan_tree_node.default_scan_object;
                        }
                    }
                }
                ScanObject::Signature(signature) => {
                    if path != &signature.path {
                        break;
                    }
                    return Some(signature);
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build() -> Result<(), ErrorTrace> {
        let mut scan_tree: ScanTree = ScanTree::new(ScanTreeType::Prefix);

        assert!(matches!(scan_tree.root_scan_object, ScanObject::None));

        let mut signatures: Vec<Arc<PathFilterSignature>> = Vec::new();
        signatures.push(Arc::new(PathFilterSignature::new(
            Path::from("/testdir1/testfile1"),
            None,
        )));
        scan_tree.build(&signatures)?;

        assert!(matches!(
            scan_tree.root_scan_object,
            ScanObject::ScanTreeNode(_)
        ));

        Ok(())
    }

    #[test]
    fn test_build_without_signatures() -> Result<(), ErrorTrace> {
        let mut scan_tree: ScanTree = ScanTree::new(ScanTreeType::Prefix);

        assert!(matches!(scan_tree.root_scan_object, ScanObject::None));

        let signatures: Vec<Arc<PathFilterSignature>> = Vec::new();
        scan_tree.build(&signatures)?;

        assert!(matches!(scan_tree.root_scan_object, ScanObject::None));

        Ok(())
    }

    #[test]
    fn test_signature() -> Result<(), ErrorTrace> {
        let mut scan_tree: ScanTree = ScanTree::new(ScanTreeType::Prefix);

        let mut signatures: Vec<Arc<PathFilterSignature>> = Vec::new();
        signatures.push(Arc::new(PathFilterSignature::new(
            Path::from("/Windows/System32/winevt/Logs/Application.evtx"),
            None,
        )));
        scan_tree.build(&signatures)?;

        let path: Path = Path::from("/Windows/System32/winevt/Logs/Application.evtx");
        let scan_result: Option<&PathFilterSignature> = scan_tree.scan_path(&path);
        assert!(scan_result.is_some());

        let path: Path = Path::from("/Windows/SoftwareDistribution/DataStore/DataStore.edb");
        let scan_result: Option<&PathFilterSignature> = scan_tree.scan_path(&path);
        assert!(scan_result.is_none());

        // TODO: add test with case folding

        Ok(())
    }
}
