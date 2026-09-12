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
use keramics_formats::{Path, PathCharacterMappings, PathComponent};

use super::enums::ScanTreeType;
use super::scan_tree::ScanTree;
use super::signature::PathFilterSignature;

/// Path filter.
pub struct PathFilter {
    /// Scan paths.
    signatures: Vec<Arc<PathFilterSignature>>,

    /// Case folding mappings.
    case_folding_mappings: Option<PathCharacterMappings>,

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
            case_folding_mappings: None,
            prefix_scan_tree: ScanTree::new(ScanTreeType::Prefix),
            suffix_scan_tree: ScanTree::new(ScanTreeType::Suffix),
        }
    }

    /// Adds a signature.
    pub fn add_signature(&mut self, signature: PathFilterSignature) {
        self.signatures.push(Arc::new(signature));
    }

    /// Builds the scan trees.
    pub fn build(&mut self) -> Result<(), ErrorTrace> {
        let mut signatures: Vec<Arc<PathFilterSignature>> = Vec::new();

        for signature in self.signatures.iter() {
            match &self.case_folding_mappings {
                Some(mappings) => match signature.new_with_case_folding(mappings) {
                    Ok(case_folded_signature) => signatures.push(Arc::new(case_folded_signature)),
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "create path filter signature with case folding"
                        );
                        return Err(error);
                    }
                },
                None => signatures.push(signature.clone()),
            }
        }
        match self.prefix_scan_tree.build(&signatures) {
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
    pub fn is_match(
        &self,
        path: &Path,
        data_fork_name: Option<&PathComponent>,
    ) -> Result<bool, ErrorTrace> {
        let (path, data_fork_name): (Path, Option<PathComponent>) =
            match &self.case_folding_mappings {
                Some(mappings) => {
                    let path: Path = match path.new_with_case_folding(mappings) {
                        Ok(case_folded_path) => case_folded_path,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to apply case folding on path"
                            );
                            return Err(error);
                        }
                    };
                    let data_fork_name: Option<PathComponent> = match data_fork_name {
                        Some(name) => match name.new_with_case_folding(mappings) {
                            Ok(case_folded_name) => Some(case_folded_name),
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    "Unable to apply case folding on data fork name"
                                );
                                return Err(error);
                            }
                        },
                        None => None,
                    };
                    (path, data_fork_name)
                }
                None => (path.clone(), data_fork_name.cloned()),
            };

        match self.prefix_scan_tree.scan_path(&path) {
            Some(signature) => Ok(data_fork_name.as_ref() == signature.data_fork_name.as_ref()),
            None => {
                // TODO: add support for suffix scan tree
                Ok(false)
            }
        }
    }

    /// Sets the case folding mappings.
    pub fn set_case_folding(&mut self, case_folding_mappings: PathCharacterMappings) {
        self.case_folding_mappings = Some(case_folding_mappings);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::Arc;

    use keramics_types::Ucs2CharacterMappings;
    use keramics_types::constants::UCS2_CASE_MAPPINGS;

    #[test]
    fn test_add_signature() -> Result<(), ErrorTrace> {
        let mut path_filter: PathFilter = PathFilter::new();

        let data_fork_name: PathComponent = PathComponent::from("rsrc");
        path_filter.add_signature(PathFilterSignature::new(
            Path::from("/Library/Caches/com.apple.Safari"),
            Some(data_fork_name.clone()),
        ));

        assert_eq!(path_filter.signatures.len(), 1);
        assert_eq!(
            path_filter.signatures[0].path,
            Path::from("/Library/Caches/com.apple.Safari")
        );
        assert_eq!(
            path_filter.signatures[0].data_fork_name,
            Some(data_fork_name)
        );
        Ok(())
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
        path_filter.add_signature(PathFilterSignature::new(
            Path::from("/Windows/SoftwareDistribution/DataStore/DataStore.edb"),
            None,
        ));
        path_filter.build()?;

        let path: Path = Path::from("/Windows/System32/winevt/Logs/Application.evtx");
        let result: bool = path_filter.is_match(&path, None)?;
        assert_eq!(result, true);

        let path: Path = Path::from("/Windows/SoftwareDistribution/DataStore/DataStore.edb");
        let result: bool = path_filter.is_match(&path, None)?;
        assert_eq!(result, true);

        let path: Path = Path::from("/WINDOWS/SoftwareDistribution/DataStore/DataStore.edb");
        let result: bool = path_filter.is_match(&path, None)?;
        assert_eq!(result, false);

        let path: Path = Path::from("/Windows/System32/winevt/Logs/Security.evtx");
        let result: bool = path_filter.is_match(&path, None)?;
        assert_eq!(result, false);

        Ok(())
    }

    #[test]
    fn test_is_match_with_data_fork_name() -> Result<(), ErrorTrace> {
        let mut path_filter: PathFilter = PathFilter::new();

        path_filter.add_signature(PathFilterSignature::new(
            Path::from("/Library/Caches/com.apple.Safari"),
            Some(PathComponent::from("rsrc")),
        ));
        path_filter.build()?;

        let path: Path = Path::from("/Library/Caches/com.apple.Safari");

        let data_fork_name: PathComponent = PathComponent::from("rsrc");
        let result: bool = path_filter.is_match(&path, Some(&data_fork_name))?;
        assert_eq!(result, true);

        let data_fork_name: PathComponent = PathComponent::from("data");
        let result: bool = path_filter.is_match(&path, Some(&data_fork_name))?;
        assert_eq!(result, false);

        let result: bool = path_filter.is_match(&path, None)?;
        assert_eq!(result, false);

        Ok(())
    }

    #[test]
    fn test_is_match_with_case_folding() -> Result<(), ErrorTrace> {
        let mut path_filter: PathFilter = PathFilter::new();

        let mappings: PathCharacterMappings = PathCharacterMappings::Ucs2(Arc::new(
            Ucs2CharacterMappings::from(UCS2_CASE_MAPPINGS.as_slice()),
        ));
        path_filter.set_case_folding(mappings);

        path_filter.add_signature(PathFilterSignature::new(
            Path::from("/Windows/System32/winevt/Logs/Application.evtx"),
            None,
        ));
        path_filter.build()?;

        let path: Path = Path::from("/windows/system32/winevt/logs/application.evtx");
        let result: bool = path_filter.is_match(&path, None)?;
        assert_eq!(result, true);

        let path: Path = Path::from("/windows/system32/winevt/logs/security.evtx");
        let result: bool = path_filter.is_match(&path, None)?;
        assert_eq!(result, false);

        Ok(())
    }

    #[test]
    fn test_is_match_without_signatures() -> Result<(), ErrorTrace> {
        let mut path_filter: PathFilter = PathFilter::new();
        path_filter.build()?;

        let path: Path = Path::from("/Library/Caches/com.apple.Safari");

        let data_fork_name: PathComponent = PathComponent::from("rsrc");
        let result: bool = path_filter.is_match(&path, Some(&data_fork_name))?;
        assert_eq!(result, false);

        Ok(())
    }
}
