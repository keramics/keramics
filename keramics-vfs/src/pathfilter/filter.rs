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
use keramics_formats::{Path, PathComponent};

use super::enums::ScanTreeType;
use super::scan_tree::ScanTree;
use super::signature::PathFilterSignature;

/// Path filter.
pub struct PathFilter {
    /// Scan paths.
    signatures: Vec<Arc<PathFilterSignature>>,

    /// Prefix scan tree.
    prefix_scan_tree: ScanTree,

    /// Suffix scan tree.
    suffix_scan_tree: ScanTree,
}

impl PathFilter {
    /// Creates a new path filter.
    pub fn new() -> Self {
        Self {
            signatures: Vec::new(),
            prefix_scan_tree: ScanTree::new(ScanTreeType::Prefix),
            suffix_scan_tree: ScanTree::new(ScanTreeType::Suffix),
        }
    }

    /// Adds a signature.
    pub fn add_signature(&mut self, signature: PathFilterSignature) {
        // TODO: handle case folding
        self.signatures.push(Arc::new(signature));
    }

    /// Builds the scan trees.
    pub fn build(&mut self) -> Result<(), ErrorTrace> {
        match self.prefix_scan_tree.build(&self.signatures) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to build prefix scan tree");
                return Err(error);
            }
        }
        // TODO: add support for suffix scan tree

        Ok(())
    }

    /// Determines whether the given path matches the filter.
    pub fn is_match(&self, path: &Path, data_fork_name: Option<&PathComponent>) -> bool {
        match self.prefix_scan_tree.scan_path(path) {
            Some(signature) => data_fork_name == signature.data_fork_name.as_ref(),
            None => {
                // TODO: add support for suffix scan tree
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_signature() {
        let mut path_filter: PathFilter = PathFilter::new();

        assert_eq!(path_filter.signatures.len(), 0);

        path_filter.add_signature(PathFilterSignature::new(
            Path::from("/Windows/System32/winevt/Logs/Application.evtx"),
            None,
        ));
        assert_eq!(path_filter.signatures.len(), 1);
    }

    #[test]
    fn test_build() -> Result<(), ErrorTrace> {
        let mut path_filter: PathFilter = PathFilter::new();

        path_filter.add_signature(PathFilterSignature::new(
            Path::from("/Windows/System32/winevt/Logs/Application.evtx"),
            None,
        ));
        path_filter.build()
    }

    #[test]
    fn test_is_match() -> Result<(), ErrorTrace> {
        let mut path_filter: PathFilter = PathFilter::new();

        path_filter.add_signature(PathFilterSignature::new(
            Path::from("/Windows/System32/winevt/Logs/Application.evtx"),
            None,
        ));
        path_filter.build()?;

        let path: Path = Path::from("/Windows/System32/winevt/Logs/Application.evtx");
        let result: bool = path_filter.is_match(&path, None);
        assert_eq!(result, true);

        let path: Path = Path::from("/Windows/SoftwareDistribution/DataStore/DataStore.edb");
        let result: bool = path_filter.is_match(&path, None);
        assert_eq!(result, false);

        // TODO: add test with case folding

        Ok(())
    }
}
